#!/bin/sh

while true
do
ps -ef | grep "./target/release/most" | grep -v "grep"
if [ "$?" -eq 1 ]
then
RUST_LOG=info nohup nice -n -20 ./target/release/most 2>> nohup.out &
echo "restart"
fi
sleep 5
done
