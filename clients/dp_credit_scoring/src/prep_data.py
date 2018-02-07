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
    num_cols = raw_data.loc[:, NUMERIC_COLS]
    data.loc[:, NUMERIC_COLS] = (num_cols - num_cols.mean(0)) / (num_cols.std(0) + 1e-8)
    data = data.assign(**{
        'PAY_DULY_%d' % i: (raw_data['PAY_%d' % i] == -1) * 1 for i in BILLS})
    data = data.rename(columns={'default payment next month': 'will_default'})
    data.columns = [colname.lower() for colname in data.columns]

    return data


def _pack_proto(api, train_inputs, train_targets, test_inputs, test_targets):
    def _mk_matrix(data):
        return api.Matrix(rows=data.shape[0],
                          cols=data.shape[1],
                          data=data.ravel().tolist())
    return api.Dataset(train_inputs=_mk_matrix(train_inputs),
                       train_targets=train_targets.tolist(),
                       test_inputs=_mk_matrix(test_inputs),
                       test_targets=test_targets.tolist())


def _split_data(data, train_frac, seed, max_samples):
    np.random.seed(seed)
    shuf_data = data.reindex(np.random.permutation(data.index))
    split_idx = int(len(data) * train_frac)

    targets = data.pop('will_default').as_matrix()
    inputs = data.as_matrix()

    max_samples = max_samples or float('inf')
    train_split = slice(0, min(max_samples, split_idx))
    test_split = slice(split_idx, min(split_idx + max_samples, len(data)))

    inputs_train = inputs[train_split]
    inputs_test = inputs[test_split]
    targets_train = targets[train_split]
    targets_test = targets[test_split]

    assert max_samples == float('inf') or (
        len(inputs_train) <= max_samples and len(inputs_test) <= max_samples and
        len(targets_train) <= max_samples and len(targets_test) <= max_samples)

    return inputs_train, targets_train, inputs_test, targets_test


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--api-proto', required=True, type=osp.abspath)
    parser.add_argument('--seed', type=int, default=42)
    parser.add_argument('--train-frac', type=float, default=0.8)
    parser.add_argument('--max-samples', type=int)
    args = parser.parse_args()

    prepped_data = _prep_data()
    split_data = _split_data(prepped_data, args.train_frac, args.seed, args.max_samples)

    api_pb2 = imp.load_source('api_pb2', args.api_proto)
    proto_data = _pack_proto(api_pb2, *split_data)

    if not sys.stdout.isatty():
        sys.stdout.write(proto_data.SerializeToString())
        sys.stdout.flush()


if __name__ == '__main__':
    main()
