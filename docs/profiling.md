# Profiling

## Non-SGX

To profile non-SGX portions of Ekiden, you can use standard tools like `valgrind`. Note that there
is a bug in older Valgrind versions, which makes it incorrectly advertise RDRAND support in CPUID
and when it is used it crashes with an illegal instruction error. For this reason be sure to use
Valgrind version 3.13 or greater which is known to work.

After installing Valgrind, you can use it as normal (e.g., for profiling the compute node):
```bash
$ valgrind \
    --tool=callgrind \
    --callgrind-out-file=callgrind.out \
    target/debug/compute target/enclave/token.signed.so
```

After the program terminates (you can interrupt it using CTRL+C), you can run the annotate tool
to get a human-readable report:
```bash
$ callgrind_annotate callgrind.out
```

## SGX

TODO
