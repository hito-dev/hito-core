#!/bin/zsh

version=1.9.1
export PATH=/opt/nordic/ncs/v$version/toolchain/bin:$PATH
#export GIT_EXEC_PATH=/opt/nordic/ncs/v$version/toolchain/Cellar/git/2.26.2/libexec/git-core 
export GIT_EXEC_PATH=/opt/nordic/ncs/v$version/toolchain/bin/git
export ZEPHYR_TOOLCHAIN_VARIANT=gnuarmemb 
export GNUARMEMB_TOOLCHAIN_PATH=/opt/nordic/ncs/v$version/toolchain
export ZEPHYR_BASE=/opt/nordic/ncs/v$version/zephyr
# source /opt/nordic/ncs/v$version/.venv/bin/activate

west config build.board nrf5340dk_nrf5340_cpuapp
