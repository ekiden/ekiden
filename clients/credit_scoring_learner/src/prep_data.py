#!/usr/bin/env python2
"""Loads, formats, and dumps the "Default of Credit Card Clients" dataset.

https://archive.ics.uci.edu/ml/datasets/default+of+credit+card+clients

Note: The first time this script is run, it will take a while to pack the data
into proto format. The serialized data are cached so future runs will be fast.
"""

import argparse
import imp
import os
from os import path as osp
import sys
import tempfile

import numpy as np
import pandas as pd


DS_URL = ('https://archive.ics.uci.edu/ml/machine-learning-databases'
          '/00350/default%20of%20credit%20card%20clients.xls')

DATA_DIR = osp.join(tempfile.gettempdir(), 'credit_scoring_data')
DATA_PROTO = osp.join(DATA_DIR, 'data.pb')
DATA_CSV = osp.join(DATA_DIR, 'data.csv')

BILLS = range(1, 7)
NUMERIC_COLS = (['LIMIT_BAL', 'AGE'] +
                ['BILL_AMT%d' % i for i in BILLS] +
                ['PAY_AMT%d' % i for i in BILLS])
INDICATOR_COLS = ['SEX', 'EDUCATION', 'MARRIAGE']


def _prep_data():
    raw_data = (pd.read_excel(DS_URL, header=1)
                .set_index('ID')
                .rename(columns={'PAY_0': 'PAY_1'}))

    data = pd.get_dummies(raw_data, columns=INDICATOR_COLS)
    data = data.assign(**{
        'PAY_DULY_%d' % i: (raw_data['PAY_%d' % i] == -1) * 1 for i in BILLS})
    data = data.rename(columns={'default payment next month': 'will_default'})
    data.columns = [colname.lower() for colname in data.columns]

    return data


def _pack_proto(proto_api, data_df):
    examples = []
    for _i, row in data_df.iterrows():
        feature = {}
        for col_name, val in row.iteritems():
            if isinstance(val, str):
                feature[col_name] = proto_api.Feature(
                    bytes_list=proto_api.BytesList(value=[val]))
            else:
                feature[col_name] = proto_api.Feature(
                    float_list=proto_api.FloatList(value=[val]))
        examples.append(proto_api.Example(
            features=proto_api.Features(feature=feature)))
    return proto_api.Examples(examples=examples)


def _split_data(data, train_frac, seed):
    np.random.seed(seed)
    shuf_data = data.reindex(np.random.permutation(data.index))
    split_idx = int(len(data) * train_frac)
    is_train = np.ones(len(data))
    is_train[split_idx:] = 0
    return shuf_data.assign(is_train=is_train)


def main():
    if osp.isfile(DATA_PROTO):
        with open(DATA_PROTO) as f_ds:
            sys.stdout.write(f_ds.read())
            sys.stdout.flush()
            return

    # exit()
    parser = argparse.ArgumentParser()
    parser.add_argument('--api-proto', required=True, type=osp.abspath)
    parser.add_argument('--seed', type=int, default=42)
    parser.add_argument('--train-frac', type=float, default=2./3.)
    args = parser.parse_args()

    if not osp.isdir(DATA_DIR):
        os.mkdir(DATA_DIR)

    prepped_data = _prep_data()
    split_data = _split_data(prepped_data, args.train_frac, args.seed)
    split_data.to_csv(DATA_CSV)

    api_pb2 = imp.load_source('api_pb2', args.api_proto)
    proto_data = _pack_proto(api_pb2, split_data)
    ser_data = proto_data.SerializeToString()

    with open(DATA_PROTO, 'w') as f_ds:
        f_ds.write(ser_data)

    if not sys.stdout.isatty():
        sys.stdout.write(ser_data)
        sys.stdout.flush()


if __name__ == '__main__':
    main()
