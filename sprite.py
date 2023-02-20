import argparse
import itertools


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('infile')

    args = parser.parse_args()

    with open(args.infile) as fh:
        contents = fh.read()
    
    lines = contents.splitlines()
    assert all([len(line) == len(lines[0]) for line in lines]), 'Lines are different lengths'

    valid_values = {'1', '2', '3', '4'}
    assert all([num in valid_values for line in lines for num in line]), f'All values must be in {valid_values!r}'

    linear_nums = [int(num) - 1 for line in lines for num in line]
    data = []
    for nums in itertools.zip_longest(linear_nums[::4], linear_nums[1::4], linear_nums[2::4], linear_nums[3::4], fillvalue=None):
        res = 0
        for i, num in enumerate(nums):
            if num is not None:
                res |= num << ((3 - i) * 2)
        data.append(res)

    print(data)
    

if __name__ == '__main__':
    main()