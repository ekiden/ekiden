#!/usr/bin/env python2
"""Evaluates the model trained on the credit card client dataset."""

import argparse
from os import path as osp
import pickle
import sys
import tempfile

import numpy as np
import pandas as pd
from sklearn import metrics


SAVE_PREDS = osp.join(tempfile.gettempdir(), 'preds.npz')


def main():
    if sys.stdin.isatty():
        pgt = np.load(SAVE_PREDS)
        preds = pgt['preds']
        ground_truth = pgt['ground_truth']
    else:
        preds, ground_truth = list(map(np.array, pickle.loads(sys.stdin.read())))
        np.savez(SAVE_PREDS, preds=preds, ground_truth=ground_truth)

    fpr, tpr, thresholds = metrics.roc_curve(ground_truth, preds)
    print("acc: %f" % ((preds > 0.5) == ground_truth).mean())
    print("AUC: %f" % metrics.auc(fpr, tpr))


if __name__ == '__main__':
    main()
