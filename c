#!/usr/bin/bash
clear 
if [[ -z $1 ]]
then 
echo "idiot! !"
else
cargo run $1
fi