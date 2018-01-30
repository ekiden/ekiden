#!/usr/bin/env python2
"""Evaluates a logistic regression model trained on the credit card client dataset."""

import argparse
import imp
from os import path as osp
import pickle
import sys
import tempfile

import numpy as np
import pandas as pd


DATA_DIR = osp.join(tempfile.gettempdir(), 'credit_scoring_data')
DATA_CSV = osp.join(DATA_DIR, 'data.csv')


def main():
    preds = np.array(pickle.loads(sys.stdin.read()))
    print(params)
    with open('preds.pkl', 'wb') as f_preds:
        pickle.dump(preds, f_preds)


if __name__ == '__main__':
    main()
