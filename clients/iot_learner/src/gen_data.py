#!/usr/bin/env python2
"""
Fetches temperature data from CIEE using XBOS.

Based heavily on
1: https://github.com/apanagopoulos/XBOS-DR/blob/master/ThermalModel.py
"""

import argparse
import contextlib
from datetime import datetime, timedelta
import imp
from os import path as osp
import sys

import numpy as np
import pandas as pd
import pytz
from xbos import get_client
from xbos.services.pundat import DataClient, timestamp, make_dataframe


CURDIR = osp.abspath(osp.dirname(__file__))

# the following constants are taken from [1]
DATE_FMT = '"%Y-%m-%d %H:%M:%S %Z"'
SE_TEMP = 'b47ba370-bceb-39cf-9552-d1225d910039'
SE_STATE = '7e543d07-16d1-32bb-94af-95a01f4675f9'
UUIDS = [SE_TEMP, SE_STATE]
INTERVAL = '15min'


class DummyFile(object):
    def write(self, x): pass

@contextlib.contextmanager
def _silence():
    save_stdout = sys.stdout
    sys.stdout = DummyFile()
    yield
    sys.stdout = save_stdout


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--api-proto', required=True, type=osp.abspath)
    args = parser.parse_args()

    api_pb2 = imp.load_source('api_pb2', args.api_proto)

    examples = []
    for i, row in _fetch_dataframe().iterrows():
        feature = {}
        for k, v in row.iteritems():
            if isinstance(v, str):
                feature[k] = api_pb2.Feature(
                    bytes_list=api_pb2.BytesList(value=[v]))
            else:
                feature[k] = api_pb2.Feature(
                    float_list=api_pb2.FloatList(value=[v]))
        examples.append(api_pb2.Example(
            features=api_pb2.Features(feature=feature)))

    sys.stdout.write(api_pb2.Examples(examples=examples).SerializeToString())
    sys.stdout.flush()


def _fetch_dataframe():
    """
    Returns a `pandas.DataFrame` with columns
        tin: the current temperature
        a: 0=noop, 1=cooling, 2=heating
        a1: is cooling?
        a2: is heating?
        next_temp: the temperature at the next time step
    """
    with _silence():
        # set $BW2_AGENT and $BW2_DEFAULT_ENTITY
        archiver = DataClient(get_client())

    now = datetime.now(pytz.timezone('America/Los_Angeles'))

    start = (now + timedelta(minutes=15)).strftime(DATE_FMT)
    end = (now - timedelta(days=1)).strftime(DATE_FMT)

    dfs = make_dataframe(archiver.window_uuids(UUIDS, end, start, INTERVAL))

    for uid, df in dfs.items():
        if uid == SE_TEMP:
            if 'mean' in df.columns:
                df = df[['mean']]
            df.columns = ['tin']
        elif uid == SE_STATE:
            if 'max' in df.columns:
                df = df[['max']]
            df.columns = ['a']
        dfs[uid] = df.resample(INTERVAL).mean()

    df = pd.concat([dframe for uid, dframe in dfs.items()], axis=1)
    df['a1'] = df.apply(lambda row: int(row['a'] > 0 and row['a'] <= 1), axis=1)
    df['a2'] = df.apply(lambda row: int(row['a'] > 1), axis=1)
    # the following are the features used by the baseline model
    df['tin'] = df['tin'].replace(to_replace=0, method='pad')
    df['tin_a1'] = df.tin * df.a1
    df['tin_a2'] = df.tin * df.a2
    df['next_temp'] = df['tin'].shift(-1)
    # the following are necessary because rulinalg complains about ill-conditioning
    # note that numpy does not have this problem
    df.tin_a1 += np.random.randn(len(df.tin)) * 1e-8
    df.tin_a2 += np.random.randn(len(df.tin)) * 1e-8
    df = df.dropna()

    return df


if __name__ == '__main__':
    main()
