The math in gf256 is very close to the hardware, and for each algorithm, there
is usually more than one way to do things.

To make things more complicated, the best implementation depends on a number of
things, such as what hardware is available (ie [carry-less multiplication
][xmul]), memory, and constant-time requirements.

To help with this, gf256 uses highly-configurable `proc_macro`s to let users
choose the best implementation for their purpose. More information about these
macros can be found in [README.md](README.md).

---

In order to choose a good default strategy for each algorithm, gf256 has a set
of rudimentary benchmarks using [Criterion][criterion]. This lets us easily
compare different implementation strategies on different hardware
configurations.

It's important to note, these aren't the the most exhaustive benchmarks, and
they haven't been ran on the most exhaustive set of hardware. Fortunately, even
with rudimentary benchmarking, there's usually a pretty clear winner for each
configuration. At least of the options available in gf256.

Feel free to run these benchmarks locally, to find the most performant
implementation on your machine:

``` bash
$ make bench
```

The following are the most recent results (which are probably already
out-of-date). Bold results indicate the fastest implementation in their
category.

[xmul]: https://en.wikipedia.org/wiki/Carry-less_product
[criterion]: https://docs.rs/criterion/latest/criterion

---

## Carry-less multiplication

|                    | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:-------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| xmul/naive_xmul    |                16.406 ns   |                        **16.139 ns** |           37.017 ns   |                     37.921 ns   |       97.731 ns   |               **97.753 ns** |       229.35 ns   |                 228.79 ns   |
| xmul/hardware_xmul |              **999.84 ps** |                          16.156 ns   |         **3.4420 ns** |                   **36.596 ns** |     **4.7512 ns** |                 97.766 ns   |     **9.3101 ns** |               **228.76 ns** |

## Galois-field multiplication

|                                 | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:--------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| gfmul/gf256_naive_mul           |                11.129 ns   |                          16.953 ns   |           21.706 ns   |                     26.184 ns   |       33.356 ns   |                 36.286 ns   |       33.851 ns   |                 56.792 ns   |
| gfmul/gf256_table_mul           |              **983.50 ps** |                        **1.0578 ns** |         **2.2841 ns** |                   **2.3177 ns** |     **3.6110 ns** |               **3.6088 ns** |       14.381 ns   |               **14.419 ns** |
| gfmul/gf256_rem_table_mul       |                984.46 ps   |                          5.0971 ns   |           2.6204 ns   |                     11.826 ns   |       5.4644 ns   |                 11.723 ns   |     **11.588 ns** |                 33.700 ns   |
| gfmul/gf256_small_rem_table_mul |                6.2360 ns   |                          10.970 ns   |           18.028 ns   |                     21.038 ns   |       18.862 ns   |                 26.444 ns   |       41.849 ns   |                 70.230 ns   |
| gfmul/gf256_barret_mul          |                2.2866 ns   |                          9.3038 ns   |           12.056 ns   |                     18.456 ns   |       10.515 ns   |                 23.392 ns   |       15.152 ns   |                 49.276 ns   |

|                                 | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53  @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:--------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|-------------------:|----------------------------:|
| gfmul/gf16_naive_mul            |                7.5132 ns   |                          12.575 ns   |           13.817 ns   |                     21.909 ns   |       24.037 ns   |                 27.576 ns   |        22.778 ns   |                 45.215 ns   |
| gfmul/gf16_table_mul            |                1.8285 ns   |                        **1.8217 ns** |           3.8362 ns   |                     3.8429 ns   |       7.1143 ns   |               **7.1034 ns** |        14.892 ns   |               **14.897 ns** |
| gfmul/gf16_rem_table_mul        |              **1.0802 ns** |                          2.8074 ns   |         **3.1999 ns** |                     5.8933 ns   |     **5.7304 ns** |                 7.8272 ns   |      **13.700 ns** |                 17.189 ns   |
| gfmul/gf16_small_rem_table_mul  |                6.4772 ns   |                          9.4698 ns   |           18.378 ns   |                     18.310 ns   |       20.017 ns   |                 20.058 ns   |        43.369 ns   |                 52.566 ns   |
| gfmul/gf16_barret_mul           |                2.3745 ns   |                          1.9874 ns   |           9.4158 ns   |                   **3.3668 ns** |       13.102 ns   |                 21.160 ns   |        18.690 ns   |                 23.715 ns   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| gfmul/gf2p16_naive_mul           |                15.953 ns   |                          26.443 ns   |           27.075 ns   |                     41.326 ns   |       40.831 ns   |                 57.402 ns   |       50.381 ns   |                 99.296 ns   |
| gfmul/gf2p16_rem_table_mul       |                6.7126 ns   |                          14.743 ns   |           12.718 ns   |                   **29.823 ns** |       18.087 ns   |               **34.586 ns** |       36.750 ns   |               **83.857 ns** |
| gfmul/gf2p16_small_rem_table_mul |                10.013 ns   |                          17.645 ns   |           24.320 ns   |                     38.890 ns   |       35.569 ns   |                 50.127 ns   |       69.866 ns   |                 119.24 ns   |
| gfmul/gf2p16_barret_mul          |              **2.2937 ns** |                        **13.455 ns** |         **8.9820 ns** |                     31.893 ns   |     **9.8849 ns** |                 53.460 ns   |     **15.426 ns** |                 108.54 ns   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| gfmul/gf2p32_naive_mul           |                24.942 ns   |                          35.281 ns   |           41.008 ns   |                     53.034 ns   |       58.916 ns   |                 101.61 ns   |       63.794 ns   |               **119.28 ns** |
| gfmul/gf2p32_rem_table_mul       |                10.687 ns   |                        **14.406 ns** |           21.535 ns   |                     34.929 ns   |       26.756 ns   |               **79.501 ns** |       59.424 ns   |                 193.97 ns   |
| gfmul/gf2p32_small_rem_table_mul |                18.864 ns   |                          19.921 ns   |           37.345 ns   |                     49.648 ns   |       54.087 ns   |                 99.291 ns   |       112.18 ns   |                 208.58 ns   |
| gfmul/gf2p32_barret_mul          |              **2.2320 ns** |                          17.901 ns   |         **9.0284 ns** |                   **33.821 ns** |     **9.6186 ns** |                 90.596 ns   |     **15.807 ns** |                 194.67 ns   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| gfmul/gf2p64_naive_mul           |                83.672 ns   |                          110.04 ns   |           129.96 ns   |                     214.52 ns   |       221.70 ns   |                 418.54 ns   |       283.20 ns   |                 667.17 ns   |
| gfmul/gf2p64_rem_table_mul       |                20.533 ns   |                        **35.096 ns** |           34.605 ns   |                     124.94 ns   |       48.117 ns   |               **249.24 ns** |       109.16 ns   |               **506.18 ns** |
| gfmul/gf2p64_small_rem_table_mul |                36.280 ns   |                          50.096 ns   |           61.355 ns   |                     152.01 ns   |       102.13 ns   |                 298.31 ns   |       213.93 ns   |                 613.42 ns   |
| gfmul/gf2p64_barret_mul          |              **1.8085 ns** |                          45.276 ns   |         **6.9333 ns** |                   **120.59 ns** |     **9.2153 ns** |                 313.76 ns   |     **15.890 ns** |                 618.90 ns   |

## Galois-field division

|                                 | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:--------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| gfdiv/gf256_naive_div           |                118.69 ns   |                          154.64 ns   |           271.21 ns   |                     324.45 ns   |       397.56 ns   |                 446.31 ns   |       440.14 ns   |                 752.00 ns   |
| gfdiv/gf256_table_div           |              **972.08 ps** |                        **957.69 ps** |         **2.3997 ns** |                   **2.6109 ns** |     **4.1131 ns** |               **4.1154 ns** |     **15.102 ns** |               **15.128 ns** |
| gfdiv/gf256_rem_table_div       |                19.690 ns   |                          82.690 ns   |           59.391 ns   |                     146.69 ns   |       95.588 ns   |                 190.76 ns   |       117.64 ns   |                 365.01 ns   |
| gfdiv/gf256_small_rem_table_div |                73.855 ns   |                          138.28 ns   |           178.03 ns   |                     272.30 ns   |       238.57 ns   |                 334.75 ns   |       461.02 ns   |                 849.28 ns   |
| gfdiv/gf256_barret_div          |                61.260 ns   |                          112.78 ns   |           221.96 ns   |                     230.49 ns   |       244.08 ns   |                 340.33 ns   |       182.47 ns   |                 738.94 ns   |

|                                | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:-------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| gfdiv/gf16_naive_div           |                31.564 ns   |                          47.578 ns   |           69.057 ns   |                     96.437 ns   |       110.71 ns   |                 140.21 ns   |       120.36 ns   |                 282.55 ns   |
| gfdiv/gf16_table_div           |              **2.1055 ns** |                        **2.0982 ns** |         **3.8425 ns** |                   **4.1275 ns** |     **6.6130 ns** |               **6.6105 ns** |     **14.999 ns** |               **14.952 ns** |
| gfdiv/gf16_rem_table_div       |                10.205 ns   |                          18.079 ns   |           33.548 ns   |                     33.175 ns   |       52.817 ns   |                 44.515 ns   |       77.164 ns   |                 102.05 ns   |
| gfdiv/gf16_small_rem_table_div |                36.017 ns   |                          47.200 ns   |           92.921 ns   |                     82.143 ns   |       122.92 ns   |                 106.78 ns   |       233.20 ns   |                 266.85 ns   |
| gfdiv/gf16_barret_div          |                27.939 ns   |                          35.534 ns   |           111.64 ns   |                     67.928 ns   |       124.02 ns   |                 117.35 ns   |       107.55 ns   |                 223.28 ns   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| gfdiv/gf2p16_naive_div           |                432.36 ns   |                          757.89 ns   |           728.90 ns   |                     1.3381 us   |       1.0561 us   |                 1.5762 us   |       1.2732 us   |                 2.8786 us   |
| gfdiv/gf2p16_rem_table_div       |                160.22 ns   |                        **406.33 ns** |         **364.29 ns** |                   **884.03 ns** |       592.46 ns   |               **1.1292 us** |       1.0873 us   |               **2.6141 us** |
| gfdiv/gf2p16_small_rem_table_div |                375.07 ns   |                          550.93 ns   |           585.41 ns   |                     1.0450 us   |       1.0087 us   |                 1.5059 us   |       1.8075 us   |                 3.2385 us   |
| gfdiv/gf2p16_barret_div          |              **149.24 ns** |                          432.70 ns   |           440.55 ns   |                     954.82 ns   |     **517.61 ns** |                 1.6336 us   |     **452.15 ns** |                 3.0219 us   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| gfdiv/gf2p32_naive_div           |                1.3246 us   |                          1.9433 us   |           2.1068 us   |                     2.8985 us   |       3.0042 us   |                 5.5931 us   |       3.2607 us   |               **6.6853 us** |
| gfdiv/gf2p32_rem_table_div       |                587.34 ns   |                        **895.70 ns** |           1.2291 us   |                   **2.1905 us** |       1.6276 us   |               **4.4786 us** |       3.2995 us   |                 9.9253 us   |
| gfdiv/gf2p32_small_rem_table_div |                1.1231 us   |                          1.5014 us   |           2.1185 us   |                     3.2455 us   |       3.3309 us   |                 6.6975 us   |       6.3128 us   |                 14.666 us   |
| gfdiv/gf2p32_barret_div          |              **275.00 ns** |                          1.2688 us   |         **917.81 ns** |                     2.5065 us   |     **1.0120 us** |                 5.7242 us   |     **848.13 ns** |                 12.123 us   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| gfdiv/gf2p64_naive_div           |                8.0257 us   |                          11.473 us   |           13.535 us   |                     25.363 us   |       21.452 us   |                 50.114 us   |       27.224 us   |                 78.147 us   |
| gfdiv/gf2p64_rem_table_div       |                2.2686 us   |                        **4.9446 us** |           4.0306 us   |                     16.621 us   |       6.0299 us   |               **34.536 us** |       13.017 us   |               **64.879 us** |
| gfdiv/gf2p64_small_rem_table_div |                4.7742 us   |                          6.9519 us   |           7.6062 us   |                     19.217 us   |       14.480 us   |                 41.218 us   |       25.220 us   |                 77.527 us   |
| gfdiv/gf2p64_barret_div          |              **395.02 ns** |                          5.9427 us   |         **1.2398 us** |                   **15.405 us** |     **1.9941 us** |                 43.758 us   |     **1.5969 us** |                 82.216 us   |

## LFSRs

|                                | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:-------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| lfsr/lfsr64_naive              |             127.34 MiB/s   |                       126.65 MiB/s   |        95.677 MiB/s   |                  89.662 MiB/s   |    71.481 MiB/s   |              71.480 MiB/s   |    52.768 MiB/s   |              52.676 MiB/s   |
| lfsr/lfsr64_divrem             |             49.718 MiB/s   |                       50.553 MiB/s   |        27.616 MiB/s   |                  27.371 MiB/s   |    17.307 MiB/s   |              17.302 MiB/s   |    12.324 MiB/s   |              12.312 MiB/s   |
| lfsr/lfsr64_table              |           **540.48 MiB/s** |                     **540.38 MiB/s** |      **414.77 MiB/s** |                **404.96 MiB/s** |  **240.76 MiB/s** |            **240.76 MiB/s** |  **167.68 MiB/s** |            **167.88 MiB/s** |
| lfsr/lfsr64_small_table        |             269.88 MiB/s   |                       267.43 MiB/s   |        244.64 MiB/s   |                  242.61 MiB/s   |    121.45 MiB/s   |              121.45 MiB/s   |    86.121 MiB/s   |              86.094 MiB/s   |
| lfsr/lfsr64_barret             |             79.492 MiB/s   |                       69.227 MiB/s   |        45.364 MiB/s   |                  40.412 MiB/s   |    30.900 MiB/s   |              20.358 MiB/s   |    21.647 MiB/s   |              11.715 MiB/s   |
| lfsr/lfsr64_table_barret       |             155.21 MiB/s   |                       43.711 MiB/s   |        49.520 MiB/s   |                  21.622 MiB/s   |    46.972 MiB/s   |              7.3621 MiB/s   |    36.694 MiB/s   |              3.1524 MiB/s   |
| lfsr/lfsr64_small_table_barret |             77.618 MiB/s   |                       21.981 MiB/s   |        25.389 MiB/s   |                  11.254 MiB/s   |    23.147 MiB/s   |              3.6297 MiB/s   |    16.867 MiB/s   |              1.5732 MiB/s   |

|                                | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:-------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| lfsr/xorshift64                |           **4.9006 GiB/s** |                     **4.9463 GiB/s** |      **2.9912 GiB/s** |                **2.9687 GiB/s** |  **2.2308 GiB/s** |            **2.2316 GiB/s** |  **1.7497 GiB/s** |            **1.7466 GiB/s** |

## CRCs

|                              | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) | Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:-----------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|------------------:|----------------------------:|------------------:|----------------------------:|
| crc/naive_crc                |             70.162 MiB/s   |                       72.168 MiB/s   |        50.933 MiB/s   |                  51.872 MiB/s   |    37.713 MiB/s   |              37.731 MiB/s   |    27.545 MiB/s   |              27.569 MiB/s   |
| crc/less_naive_crc           |             76.072 MiB/s   |                       77.650 MiB/s   |        50.547 MiB/s   |                  50.812 MiB/s   |    39.730 MiB/s   |              39.721 MiB/s   |    26.656 MiB/s   |              26.766 MiB/s   |
| crc/word_less_naive_crc      |             143.77 MiB/s   |                       144.38 MiB/s   |        91.878 MiB/s   |                  103.83 MiB/s   |    73.326 MiB/s   |              73.330 MiB/s   |    61.919 MiB/s   |              61.920 MiB/s   |
| crc/table_crc                |             476.43 MiB/s   |                     **475.45 MiB/s** |      **412.85 MiB/s** |                **444.69 MiB/s** |    213.55 MiB/s   |            **213.32 MiB/s** |    148.11 MiB/s   |            **147.55 MiB/s** |
| crc/small_table_crc          |             213.63 MiB/s   |                       215.92 MiB/s   |        186.30 MiB/s   |                  199.50 MiB/s   |    95.077 MiB/s   |              95.093 MiB/s   |    80.944 MiB/s   |              81.181 MiB/s   |
| crc/barret_crc               |             147.86 MiB/s   |                       50.603 MiB/s   |        45.476 MiB/s   |                  27.352 MiB/s   |    44.161 MiB/s   |              16.026 MiB/s   |    34.457 MiB/s   |              6.8674 MiB/s   |
| crc/word_barret_crc          |           **664.75 MiB/s** |                       209.72 MiB/s   |        208.67 MiB/s   |                  108.86 MiB/s   |  **213.85 MiB/s** |              67.776 MiB/s   |  **362.81 MiB/s** |              27.877 MiB/s   |
| crc/reversed_barret_crc      |             127.57 MiB/s   |                       50.016 MiB/s   |        39.038 MiB/s   |                  24.993 MiB/s   |    38.342 MiB/s   |              12.993 MiB/s   |    24.023 MiB/s   |              5.6548 MiB/s   |
| crc/word_reversed_barret_crc |             592.13 MiB/s   |                       214.57 MiB/s   |        194.16 MiB/s   |                  110.00 MiB/s   |    191.32 MiB/s   |              56.501 MiB/s   |    166.23 MiB/s   |              22.548 MiB/s   |


