#!/usr/bin/env python2
"""Loads, formats, and dumps the "Default of Credit Card Clients" dataset.

https://archive.ics.uci.edu/ml/datasets/default+of+credit+card+clients

Note: The first time this script is run, it will take a while to pack the data
into proto format. The serialized data are cached so future runs will be fast.
"""

import argparse
import imp
from os import path as osp
import sys
import tempfile

import pandas as pd

DS_URL = 'https://archive.ics.uci.edu/ml/machine-learning-databases/00350/default%20of%20credit%20card%20clients.xls'
PROTO_CACHE = osp.join(tempfile.gettempdir(), 'data.pb')
PREPPED_DATA_CSV = osp.join(tempfile.gettempdir(), 'data.csv')

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


def _pack_proto(proto_api, df):
    examples = []
    for _i, row in df.iterrows():
        feature = {}
        for col_name, v in row.iteritems():
            if isinstance(v, str):
                feature[col_name] = proto_api.Feature(
                    bytes_list=proto_api.BytesList(value=[v]))
            else:
                feature[col_name] = proto_api.Feature(
                    float_list=proto_api.FloatList(value=[v]))
        examples.append(proto_api.Example(
            features=proto_api.Features(feature=feature)))
    return proto_api.Examples(examples=examples)


def main():
    if osp.isfile(PROTO_CACHE):
        with open(PROTO_CACHE) as f_ds:
            sys.stdout.write(f_ds.read())
            sys.stdout.flush()
            return

    parser = argparse.ArgumentParser()
    parser.add_argument('--api-proto', required=True, type=osp.abspath)
    args = parser.parse_args()

    api_pb2 = imp.load_source('api_pb2', args.api_proto)

    data = _prep_data()
    data.to_csv(PREPPED_DATA_CSV)

    proto_data = _pack_proto(api_pb2, data)
    ser_data = proto_data.SerializeToString()

    with open(PROTO_CACHE, 'w') as f_ds:
        f_ds.write(ser_data)

    exit()
    sys.stdout.write(ser_data)
    sys.stdout.flush()


if __name__ == '__main__':
    main()
