#!/bin/bash

case $1 in
    -r|--reset)
        exec ./tests/scripts/on_localnet.sh start-new-validator
    ;;
esac

exec ./tests/scripts/on_localnet.sh resume-validator