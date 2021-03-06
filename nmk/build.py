#!/usr/bin/env python3
# This script support python 3.6 and later

import argparse
import logging
import lzma
import os
import re
import subprocess
from pathlib import Path
from shutil import copyfile

SCRIPT_DIR = Path(os.path.realpath(__file__)).parent
GIT_ROOT_DIR = SCRIPT_DIR
DIST_DIR = GIT_ROOT_DIR / 'dist'
# Incremental build won't work if we use same target directory
TARGET_DIR = GIT_ROOT_DIR / 'target.cross'

TARGET_TRIPLE = {
    'amd64': 'x86_64-unknown-linux-musl',
    'arm64': 'aarch64-unknown-linux-musl',
    # arm build use non hard-float to maximize compatibility in one binary
    'arm': 'arm-unknown-linux-musleabi',
    'armv7': 'armv7-unknown-linux-musleabihf',
}


class Opt:
    def __init__(self):
        self.args = []
        self.dist = False
        self.lto = False
        self.strip = False
        self.target = None
        self.verbosity = 0

    @classmethod
    def from_args(cls):
        args = build_parser().parse_args()
        self = cls()
        self.args = args.args
        self.dist = args.dist
        self.lto = args.lto
        self.strip = args.strip
        self.target = args.target
        self.verbosity = args.verbosity
        return self


def build_parser():
    parser = argparse.ArgumentParser(prog='build.py')
    parser.add_argument('-v', '--verbose',
                        dest='verbosity',
                        action='count',
                        default=0,
                        help='Request verbose logging')
    parser.add_argument('--lto',
                        dest='lto',
                        action='store_true',
                        default=False,
                        help="Sets link-time optimization to true")
    parser.add_argument('--strip',
                        dest='strip',
                        action='store_true',
                        default=False,
                        help="Strip build")
    parser.add_argument('--dist',
                        dest='dist',
                        action='store_true',
                        default=False,
                        help="Copy to dist/")
    parser.add_argument('--target',
                        dest='target',
                        default='amd64',
                        choices=TARGET_TRIPLE.keys(),
                        help="Set build target")
    parser.add_argument('args', nargs=argparse.REMAINDER)
    return parser


def clean_package(target):
    """
    Clean our package before build

    We need to do this to get correct build time & commit id embedded in binary.
    """
    args = ['cargo', 'clean', '--release', '--package', 'nmk', '--target', target, '--target-dir', str(TARGET_DIR)]
    logging.info("Cleaning packages")
    logging.debug("cmd: %s", " ".join(args))
    subprocess.call(args)


def build_rust_flags(strip):
    flags = []
    if strip:
        flags += ['-C', 'link-arg=-s']
    return " ".join(flags)


def build_release(target, strip=False, lto=False, commit_id=None):
    rust_flags = build_rust_flags(strip=strip)
    env = {
        'RUSTFLAGS': rust_flags
    }
    if lto:
        env['CARGO_PROFILE_RELEASE_LTO'] = 'true'
    if commit_id:
        env['GIT_SHORT_SHA'] = commit_id
    args = ['cross', 'build', '--release', '--target', target, '--target-dir', str(TARGET_DIR)]
    logging.info("Building %s target", target)
    logging.debug("env: %s", env)
    logging.debug("cmd: %s", " ".join(args))
    exit_code = subprocess.call(args, env=dict(os.environ.copy(), **env))
    if exit_code != 0:
        exit(exit_code)


def get_version_from_manifest(manifest_path):
    prog = re.compile(r'^version\s*=\s*"(.*)"')
    with open(manifest_path, 'rt') as f:
        for line in f:
            m = prog.match(line)
            if m is not None:
                return m.groups()[0]
    return None


def get_release_dir(target_triple):
    return TARGET_DIR / target_triple / 'release'


def get_build_commit_id():
    args = ['git', 'rev-parse', '--short', 'HEAD']
    try:
        return subprocess.check_output(args).decode().strip()
    except subprocess.SubprocessError:
        return None


def dist(target, release_dir):
    for binary in ('nmk', 'nmkup'):
        data = open(release_dir / binary, mode='rb').read()
        with lzma.open(DIST_DIR / f'{binary}-{target}.xz', mode='wb') as f:
            f.write(data)
    copyfile(release_dir / 'nmkup', str(DIST_DIR / f'nmkup-{target}'))


def setup_logging(verbosity):
    level = logging.DEBUG if verbosity > 0 else logging.INFO
    logging.basicConfig(format='%(message)s', level=level)


def main():
    opt = Opt.from_args()
    setup_logging(opt.verbosity)
    target = TARGET_TRIPLE.get(opt.target)
    commit_id = get_build_commit_id()
    clean_package(target)
    build_release(target, lto=opt.lto, commit_id=commit_id, strip=opt.strip)
    release_dir = get_release_dir(target)
    DIST_DIR.mkdir(exist_ok=True)
    if opt.dist:
        dist(target=target, release_dir=release_dir)


if __name__ == '__main__':
    main()
