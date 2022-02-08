#!/bin/bash

if [[ $OSTYPE == 'darwin'* ]]; then
  cp electron/node_modules/electron-holochain/binaries/mac/lair-keystore electron/binaries/lair-keystore
  cp electron/node_modules/electron-holochain/binaries/mac/holochain-runner electron/binaries/holochain-runner
elif [[ $OSTYPE == 'linux-gnu'* ]]; then
  cp electron/node_modules/electron-holochain/binaries/linux/lair-keystore electron/binaries/lair-keystore
  cp electron/node_modules/electron-holochain/binaries/linux/holochain-runner electron/binaries/holochain-runner
elif [[ $OSTYPE == "cygwin" ]]; then
  # POSIX compatibility layer and Linux environment emulation for Windows
  cp electron/node_modules/electron-holochain/binaries/windows/lair-keystore.exe electron/binaries/lair-keystore.exe
  cp electron/node_modules/electron-holochain/binaries/windows/holochain-runner.exe electron/binaries/holochain-runner.exe
elif [[ $OSTYPE == "msys" ]]; then
  # Lightweight shell and GNU utilities compiled for Windows (part of MinGW)
  cp electron/node_modules/electron-holochain/binaries/windows/lair-keystore.exe electron/binaries/lair-keystore.exe
  cp electron/node_modules/electron-holochain/binaries/windows/holochain-runner.exe electron/binaries/holochain-runner.exe
fi
