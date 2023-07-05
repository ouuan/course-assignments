#!/usr/bin/env python3
import sys

output = sys.argv[1]
answer = sys.argv[2]

output_number = float(open(output, 'r').read().strip())
answer_number = float(open(answer, 'r').read().strip())

if abs((output_number - answer_number) / answer_number) < 0.1:
    print('Accepted')
    print('The error is within bounds')
else:
    print('Wrong Answer')
    print('The error is beyond bounds')
