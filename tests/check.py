#!/bin/python3

"""
This file is useful for manual testing of the buffering behavior
of estream. Run with `./tests/check.py | ./target/debug/estream`
and check that estream prints each `.` as it is printed by python
rather than printing the entire line at once.
"""

import time

print("testing..", end="", flush=True)

time.sleep(1)

for i in range(5):
    print(".", end="", flush=True)
    time.sleep(1)
    
print("done")
