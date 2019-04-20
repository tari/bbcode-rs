import os
import shutil
import subprocess
from distutils.command.build_ext import build_ext
from setuptools import setup


class BuildLibraryCommand(build_ext):
    description = 'build Rust extension library'
    user_options = [
        ('rust-profile=', None, 'Cargo profile to use for Rust code'),
    ]

    cargo_manifest_dir = os.path.join('..', '..')

    def initialize_options(self):
        self.rust_profile = 'release'

    def finalize_options(self):
        assert self.rust_profile in ('release', 'dev'), \
                'Rust profile must be release or dev'

    def run(self):
        subprocess.check_call(['cargo', 'build', '--lib',
                               '--' + self.rust_profile])
        shutil.copy(
                os.path.join(self.cargo_manifest_dir, 'target', self.rust_profile, 'libbbcode.so'),
                '.')

setup(
    name="bbcode",
    py_modules=["bbcode"],
    include_package_data=True,
    data_files=[
        ('', ['libbbcode.so']),
    ],
    cmdclass={
        'build_ext': BuildLibraryCommand,
    }
)
