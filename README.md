This repository contains code needed to benchmark metrics for the [aarch64-air-lifter](https://github.com/TUM-DSE/aarch64-air-lifter)

Running the main file will create a csv-file that will contain

- the amount of generated BasicBlocks
- the amount of AIR instructions needed to translate an AArch64 instruction
- execution time of the translation of an AIR instruction

To calculate these metrics, this project executes certain tests within the aarch64-air-lifter which is why you need to clone the aarch64-air-lifter.
The following steps are required:

- Set the default args for the CheckInstructionArgs struct in the lifter (in the tests/common/lib.rs file) to:
  print_to_std = false
  debug_true
- Set the project path variable in the main.rs file to the directory where the aarch64-air-lifter is located in
- cargo build
