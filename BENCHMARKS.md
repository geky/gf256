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

|                    | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |  Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |  Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:-------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|-------------------:|----------------------------:|-------------------:|----------------------------:|
| xmul/naive_xmul    |           16.406&nbsp;ns   |                   **16.139&nbsp;ns** |      37.017&nbsp;ns   |                37.921&nbsp;ns   |   97.731&nbsp;ns   |          **97.753&nbsp;ns** |   229.35&nbsp;ns   |            228.79&nbsp;ns   |
| xmul/hardware_xmul |         **999.84&nbsp;ps** |                     16.156&nbsp;ns   |    **3.4420&nbsp;ns** |              **36.596&nbsp;ns** | **4.7512&nbsp;ns** |            97.766&nbsp;ns   | **9.3101&nbsp;ns** |          **228.76&nbsp;ns** |

## Galois-field multiplication

|                                 | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |  Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |  Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:--------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|-------------------:|----------------------------:|-------------------:|----------------------------:|
| gfmul/gf256_naive_mul           |           11.129&nbsp;ns   |                     16.953&nbsp;ns   |      21.706&nbsp;ns   |                26.184&nbsp;ns   |   33.356&nbsp;ns   |            36.286&nbsp;ns   |   33.851&nbsp;ns   |            56.792&nbsp;ns   |
| gfmul/gf256_table_mul           |         **983.50&nbsp;ps** |                   **1.0578&nbsp;ns** |    **2.2841&nbsp;ns** |              **2.3177&nbsp;ns** | **3.6110&nbsp;ns** |          **3.6088&nbsp;ns** |   14.381&nbsp;ns   |          **14.419&nbsp;ns** |
| gfmul/gf256_rem_table_mul       |           984.46&nbsp;ps   |                     5.0971&nbsp;ns   |      2.6204&nbsp;ns   |                11.826&nbsp;ns   |   5.4644&nbsp;ns   |            11.723&nbsp;ns   | **11.588&nbsp;ns** |            33.700&nbsp;ns   |
| gfmul/gf256_small_rem_table_mul |           6.2360&nbsp;ns   |                     10.970&nbsp;ns   |      18.028&nbsp;ns   |                21.038&nbsp;ns   |   18.862&nbsp;ns   |            26.444&nbsp;ns   |   41.849&nbsp;ns   |            70.230&nbsp;ns   |
| gfmul/gf256_barret_mul          |           2.2866&nbsp;ns   |                     9.3038&nbsp;ns   |      12.056&nbsp;ns   |                18.456&nbsp;ns   |   10.515&nbsp;ns   |            23.392&nbsp;ns   |   15.152&nbsp;ns   |            49.276&nbsp;ns   |

|                                 | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |  Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |  Arm A53  @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:--------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|-------------------:|----------------------------:|--------------------:|----------------------------:|
| gfmul/gf16_naive_mul            |           7.5132&nbsp;ns   |                     12.575&nbsp;ns   |      13.817&nbsp;ns   |                21.909&nbsp;ns   |   24.037&nbsp;ns   |            27.576&nbsp;ns   |    22.778&nbsp;ns   |            45.215&nbsp;ns   |
| gfmul/gf16_table_mul            |           1.8285&nbsp;ns   |                   **1.8217&nbsp;ns** |      3.8362&nbsp;ns   |                3.8429&nbsp;ns   |   7.1143&nbsp;ns   |          **7.1034&nbsp;ns** |    14.892&nbsp;ns   |          **14.897&nbsp;ns** |
| gfmul/gf16_rem_table_mul        |         **1.0802&nbsp;ns** |                     2.8074&nbsp;ns   |    **3.1999&nbsp;ns** |                5.8933&nbsp;ns   | **5.7304&nbsp;ns** |            7.8272&nbsp;ns   |  **13.700&nbsp;ns** |            17.189&nbsp;ns   |
| gfmul/gf16_small_rem_table_mul  |           6.4772&nbsp;ns   |                     9.4698&nbsp;ns   |      18.378&nbsp;ns   |                18.310&nbsp;ns   |   20.017&nbsp;ns   |            20.058&nbsp;ns   |    43.369&nbsp;ns   |            52.566&nbsp;ns   |
| gfmul/gf16_barret_mul           |           2.3745&nbsp;ns   |                     1.9874&nbsp;ns   |      9.4158&nbsp;ns   |              **3.3668&nbsp;ns** |   13.102&nbsp;ns   |            21.160&nbsp;ns   |    18.690&nbsp;ns   |            23.715&nbsp;ns   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |  Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |  Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|-------------------:|----------------------------:|-------------------:|----------------------------:|
| gfmul/gf2p16_naive_mul           |           15.953&nbsp;ns   |                     26.443&nbsp;ns   |      27.075&nbsp;ns   |                41.326&nbsp;ns   |   40.831&nbsp;ns   |            57.402&nbsp;ns   |   50.381&nbsp;ns   |            99.296&nbsp;ns   |
| gfmul/gf2p16_rem_table_mul       |           6.7126&nbsp;ns   |                     14.743&nbsp;ns   |      12.718&nbsp;ns   |              **29.823&nbsp;ns** |   18.087&nbsp;ns   |          **34.586&nbsp;ns** |   36.750&nbsp;ns   |          **83.857&nbsp;ns** |
| gfmul/gf2p16_small_rem_table_mul |           10.013&nbsp;ns   |                     17.645&nbsp;ns   |      24.320&nbsp;ns   |                38.890&nbsp;ns   |   35.569&nbsp;ns   |            50.127&nbsp;ns   |   69.866&nbsp;ns   |            119.24&nbsp;ns   |
| gfmul/gf2p16_barret_mul          |         **2.2937&nbsp;ns** |                   **13.455&nbsp;ns** |    **8.9820&nbsp;ns** |                31.893&nbsp;ns   | **9.8849&nbsp;ns** |            53.460&nbsp;ns   | **15.426&nbsp;ns** |            108.54&nbsp;ns   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |  Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) | Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|-------------------:|----------------------------:|-------------------:|----------------------------:|
| gfmul/gf2p32_naive_mul           |           24.942&nbsp;ns   |                     35.281&nbsp;ns   |      41.008&nbsp;ns   |                53.034&nbsp;ns   |   58.916&nbsp;ns   |            101.61&nbsp;ns   |   63.794&nbsp;ns   |          **119.28&nbsp;ns** |
| gfmul/gf2p32_rem_table_mul       |           10.687&nbsp;ns   |                   **14.406&nbsp;ns** |      21.535&nbsp;ns   |                34.929&nbsp;ns   |   26.756&nbsp;ns   |          **79.501&nbsp;ns** |   59.424&nbsp;ns   |            193.97&nbsp;ns   |
| gfmul/gf2p32_small_rem_table_mul |           18.864&nbsp;ns   |                     19.921&nbsp;ns   |      37.345&nbsp;ns   |                49.648&nbsp;ns   |   54.087&nbsp;ns   |            99.291&nbsp;ns   |   112.18&nbsp;ns   |            208.58&nbsp;ns   |
| gfmul/gf2p32_barret_mul          |         **2.2320&nbsp;ns** |                     17.901&nbsp;ns   |    **9.0284&nbsp;ns** |              **33.821&nbsp;ns** | **9.6186&nbsp;ns** |            90.596&nbsp;ns   | **15.807&nbsp;ns** |            194.67&nbsp;ns   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |  Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |  Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|-------------------:|----------------------------:|-------------------:|----------------------------:|
| gfmul/gf2p64_naive_mul           |           83.672&nbsp;ns   |                     110.04&nbsp;ns   |      129.96&nbsp;ns   |                214.52&nbsp;ns   |   221.70&nbsp;ns   |            418.54&nbsp;ns   |   283.20&nbsp;ns   |            667.17&nbsp;ns   |
| gfmul/gf2p64_rem_table_mul       |           20.533&nbsp;ns   |                   **35.096&nbsp;ns** |      34.605&nbsp;ns   |                124.94&nbsp;ns   |   48.117&nbsp;ns   |          **249.24&nbsp;ns** |   109.16&nbsp;ns   |          **506.18&nbsp;ns** |
| gfmul/gf2p64_small_rem_table_mul |           36.280&nbsp;ns   |                     50.096&nbsp;ns   |      61.355&nbsp;ns   |                152.01&nbsp;ns   |   102.13&nbsp;ns   |            298.31&nbsp;ns   |   213.93&nbsp;ns   |            613.42&nbsp;ns   |
| gfmul/gf2p64_barret_mul          |         **1.8085&nbsp;ns** |                     45.276&nbsp;ns   |    **6.9333&nbsp;ns** |              **120.59&nbsp;ns** | **9.2153&nbsp;ns** |            313.76&nbsp;ns   | **15.890&nbsp;ns** |            618.90&nbsp;ns   |

## Galois-field division

|                                 | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |  Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |  Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:--------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|-------------------:|----------------------------:|-------------------:|----------------------------:|
| gfdiv/gf256_naive_div           |           118.69&nbsp;ns   |                     154.64&nbsp;ns   |      271.21&nbsp;ns   |                324.45&nbsp;ns   |   397.56&nbsp;ns   |            446.31&nbsp;ns   |   440.14&nbsp;ns   |            752.00&nbsp;ns   |
| gfdiv/gf256_table_div           |         **972.08&nbsp;ps** |                   **957.69&nbsp;ps** |    **2.3997&nbsp;ns** |              **2.6109&nbsp;ns** | **4.1131&nbsp;ns** |          **4.1154&nbsp;ns** | **15.102&nbsp;ns** |          **15.128&nbsp;ns** |
| gfdiv/gf256_rem_table_div       |           19.690&nbsp;ns   |                     82.690&nbsp;ns   |      59.391&nbsp;ns   |                146.69&nbsp;ns   |   95.588&nbsp;ns   |            190.76&nbsp;ns   |   117.64&nbsp;ns   |            365.01&nbsp;ns   |
| gfdiv/gf256_small_rem_table_div |           73.855&nbsp;ns   |                     138.28&nbsp;ns   |      178.03&nbsp;ns   |                272.30&nbsp;ns   |   238.57&nbsp;ns   |            334.75&nbsp;ns   |   461.02&nbsp;ns   |            849.28&nbsp;ns   |
| gfdiv/gf256_barret_div          |           61.260&nbsp;ns   |                     112.78&nbsp;ns   |      221.96&nbsp;ns   |                230.49&nbsp;ns   |   244.08&nbsp;ns   |            340.33&nbsp;ns   |   182.47&nbsp;ns   |            738.94&nbsp;ns   |

|                                | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |  Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |  Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:-------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|-------------------:|----------------------------:|-------------------:|----------------------------:|
| gfdiv/gf16_naive_div           |           31.564&nbsp;ns   |                     47.578&nbsp;ns   |      69.057&nbsp;ns   |                96.437&nbsp;ns   |   110.71&nbsp;ns   |            140.21&nbsp;ns   |   120.36&nbsp;ns   |            282.55&nbsp;ns   |
| gfdiv/gf16_table_div           |         **2.1055&nbsp;ns** |                   **2.0982&nbsp;ns** |    **3.8425&nbsp;ns** |              **4.1275&nbsp;ns** | **6.6130&nbsp;ns** |          **6.6105&nbsp;ns** | **14.999&nbsp;ns** |          **14.952&nbsp;ns** |
| gfdiv/gf16_rem_table_div       |           10.205&nbsp;ns   |                     18.079&nbsp;ns   |      33.548&nbsp;ns   |                33.175&nbsp;ns   |   52.817&nbsp;ns   |            44.515&nbsp;ns   |   77.164&nbsp;ns   |            102.05&nbsp;ns   |
| gfdiv/gf16_small_rem_table_div |           36.017&nbsp;ns   |                     47.200&nbsp;ns   |      92.921&nbsp;ns   |                82.143&nbsp;ns   |   122.92&nbsp;ns   |            106.78&nbsp;ns   |   233.20&nbsp;ns   |            266.85&nbsp;ns   |
| gfdiv/gf16_barret_div          |           27.939&nbsp;ns   |                     35.534&nbsp;ns   |      111.64&nbsp;ns   |                67.928&nbsp;ns   |   124.02&nbsp;ns   |            117.35&nbsp;ns   |   107.55&nbsp;ns   |            223.28&nbsp;ns   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |  Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |  Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|-------------------:|----------------------------:|-------------------:|----------------------------:|
| gfdiv/gf2p16_naive_div           |           432.36&nbsp;ns   |                     757.89&nbsp;ns   |      728.90&nbsp;ns   |                1.3381&nbsp;us   |   1.0561&nbsp;us   |            1.5762&nbsp;us   |   1.2732&nbsp;us   |            2.8786&nbsp;us   |
| gfdiv/gf2p16_rem_table_div       |           160.22&nbsp;ns   |                   **406.33&nbsp;ns** |    **364.29&nbsp;ns** |              **884.03&nbsp;ns** |   592.46&nbsp;ns   |          **1.1292&nbsp;us** |   1.0873&nbsp;us   |          **2.6141&nbsp;us** |
| gfdiv/gf2p16_small_rem_table_div |           375.07&nbsp;ns   |                     550.93&nbsp;ns   |      585.41&nbsp;ns   |                1.0450&nbsp;us   |   1.0087&nbsp;us   |            1.5059&nbsp;us   |   1.8075&nbsp;us   |            3.2385&nbsp;us   |
| gfdiv/gf2p16_barret_div          |         **149.24&nbsp;ns** |                     432.70&nbsp;ns   |      440.55&nbsp;ns   |                954.82&nbsp;ns   | **517.61&nbsp;ns** |            1.6336&nbsp;us   | **452.15&nbsp;ns** |            3.0219&nbsp;us   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |  Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |  Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|-------------------:|----------------------------:|-------------------:|----------------------------:|
| gfdiv/gf2p32_naive_div           |           1.3246&nbsp;us   |                     1.9433&nbsp;us   |      2.1068&nbsp;us   |                2.8985&nbsp;us   |   3.0042&nbsp;us   |            5.5931&nbsp;us   |   3.2607&nbsp;us   |          **6.6853&nbsp;us** |
| gfdiv/gf2p32_rem_table_div       |           587.34&nbsp;ns   |                   **895.70&nbsp;ns** |      1.2291&nbsp;us   |              **2.1905&nbsp;us** |   1.6276&nbsp;us   |          **4.4786&nbsp;us** |   3.2995&nbsp;us   |            9.9253&nbsp;us   |
| gfdiv/gf2p32_small_rem_table_div |           1.1231&nbsp;us   |                     1.5014&nbsp;us   |      2.1185&nbsp;us   |                3.2455&nbsp;us   |   3.3309&nbsp;us   |            6.6975&nbsp;us   |   6.3128&nbsp;us   |            14.666&nbsp;us   |
| gfdiv/gf2p32_barret_div          |         **275.00&nbsp;ns** |                     1.2688&nbsp;us   |    **917.81&nbsp;ns** |                2.5065&nbsp;us   | **1.0120&nbsp;us** |            5.7242&nbsp;us   | **848.13&nbsp;ns** |            12.123&nbsp;us   |

|                                  | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |  Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |  Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:---------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|-------------------:|----------------------------:|-------------------:|----------------------------:|
| gfdiv/gf2p64_naive_div           |           8.0257&nbsp;us   |                     11.473&nbsp;us   |      13.535&nbsp;us   |                25.363&nbsp;us   |   21.452&nbsp;us   |            50.114&nbsp;us   |   27.224&nbsp;us   |            78.147&nbsp;us   |
| gfdiv/gf2p64_rem_table_div       |           2.2686&nbsp;us   |                   **4.9446&nbsp;us** |      4.0306&nbsp;us   |                16.621&nbsp;us   |   6.0299&nbsp;us   |          **34.536&nbsp;us** |   13.017&nbsp;us   |          **64.879&nbsp;us** |
| gfdiv/gf2p64_small_rem_table_div |           4.7742&nbsp;us   |                     6.9519&nbsp;us   |      7.6062&nbsp;us   |                19.217&nbsp;us   |   14.480&nbsp;us   |            41.218&nbsp;us   |   25.220&nbsp;us   |            77.527&nbsp;us   |
| gfdiv/gf2p64_barret_div          |         **395.02&nbsp;ns** |                     5.9427&nbsp;us   |    **1.2398&nbsp;us** |              **15.405&nbsp;us** | **1.9941&nbsp;us** |            43.758&nbsp;us   | **1.5969&nbsp;us** |            82.216&nbsp;us   |

## LFSRs

|                                | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |     Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |     Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:-------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|----------------------:|----------------------------:|----------------------:|----------------------------:|
| lfsr/lfsr64_naive              |        127.34&nbsp;MiB/s   |                  126.65&nbsp;MiB/s   |   95.677&nbsp;MiB/s   |             89.662&nbsp;MiB/s   |   71.481&nbsp;MiB/s   |         71.480&nbsp;MiB/s   |   52.768&nbsp;MiB/s   |         52.676&nbsp;MiB/s   |
| lfsr/lfsr64_divrem             |        49.718&nbsp;MiB/s   |                  50.553&nbsp;MiB/s   |   27.616&nbsp;MiB/s   |             27.371&nbsp;MiB/s   |   17.307&nbsp;MiB/s   |         17.302&nbsp;MiB/s   |   12.324&nbsp;MiB/s   |         12.312&nbsp;MiB/s   |
| lfsr/lfsr64_table              |      **540.48&nbsp;MiB/s** |                **540.38&nbsp;MiB/s** | **414.77&nbsp;MiB/s** |           **404.96&nbsp;MiB/s** | **240.76&nbsp;MiB/s** |       **240.76&nbsp;MiB/s** | **167.68&nbsp;MiB/s** |       **167.88&nbsp;MiB/s** |
| lfsr/lfsr64_small_table        |        269.88&nbsp;MiB/s   |                  267.43&nbsp;MiB/s   |   244.64&nbsp;MiB/s   |             242.61&nbsp;MiB/s   |   121.45&nbsp;MiB/s   |         121.45&nbsp;MiB/s   |   86.121&nbsp;MiB/s   |         86.094&nbsp;MiB/s   |
| lfsr/lfsr64_barret             |        79.492&nbsp;MiB/s   |                  69.227&nbsp;MiB/s   |   45.364&nbsp;MiB/s   |             40.412&nbsp;MiB/s   |   30.900&nbsp;MiB/s   |         20.358&nbsp;MiB/s   |   21.647&nbsp;MiB/s   |         11.715&nbsp;MiB/s   |
| lfsr/lfsr64_table_barret       |        155.21&nbsp;MiB/s   |                  43.711&nbsp;MiB/s   |   49.520&nbsp;MiB/s   |             21.622&nbsp;MiB/s   |   46.972&nbsp;MiB/s   |         7.3621&nbsp;MiB/s   |   36.694&nbsp;MiB/s   |         3.1524&nbsp;MiB/s   |
| lfsr/lfsr64_small_table_barret |        77.618&nbsp;MiB/s   |                  21.981&nbsp;MiB/s   |   25.389&nbsp;MiB/s   |             11.254&nbsp;MiB/s   |   23.147&nbsp;MiB/s   |         3.6297&nbsp;MiB/s   |   16.867&nbsp;MiB/s   |         1.5732&nbsp;MiB/s   |

|                                | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |     Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |     Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:-------------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|----------------------:|----------------------------:|----------------------:|----------------------------:|
| lfsr/xorshift64                |      **4.9006&nbsp;GiB/s** |                **4.9463&nbsp;GiB/s** | **2.9912&nbsp;GiB/s** |           **2.9687&nbsp;GiB/s** | **2.2308&nbsp;GiB/s** |       **2.2316&nbsp;GiB/s** | **1.7497&nbsp;GiB/s** |       **1.7466&nbsp;GiB/s** |

## CRCs

|                              | AMD Ryzen-3975WX @ 6.3 GHz | AMD Ryzen-3975WX @ 6.3 GHz (no-xmul) | AMD FX-6300 @ 3.5 GHz | AMD FX-6300 @ 3.5 GHz (no-xmul) |     Arm A72 @ 1.8 GHz | Arm A72 @ 1.8 GHz (no-xmul) |     Arm A53 @ 1.4 GHz | Arm A53 @ 1.4 GHz (no-xmul) |
|:-----------------------------|---------------------------:|-------------------------------------:|----------------------:|--------------------------------:|----------------------:|----------------------------:|----------------------:|----------------------------:|
| crc/naive_crc                |        70.162&nbsp;MiB/s   |                  72.168&nbsp;MiB/s   |   50.933&nbsp;MiB/s   |             51.872&nbsp;MiB/s   |   37.713&nbsp;MiB/s   |         37.731&nbsp;MiB/s   |   27.545&nbsp;MiB/s   |         27.569&nbsp;MiB/s   |
| crc/less_naive_crc           |        76.072&nbsp;MiB/s   |                  77.650&nbsp;MiB/s   |   50.547&nbsp;MiB/s   |             50.812&nbsp;MiB/s   |   39.730&nbsp;MiB/s   |         39.721&nbsp;MiB/s   |   26.656&nbsp;MiB/s   |         26.766&nbsp;MiB/s   |
| crc/word_less_naive_crc      |        143.77&nbsp;MiB/s   |                  144.38&nbsp;MiB/s   |   91.878&nbsp;MiB/s   |             103.83&nbsp;MiB/s   |   73.326&nbsp;MiB/s   |         73.330&nbsp;MiB/s   |   61.919&nbsp;MiB/s   |         61.920&nbsp;MiB/s   |
| crc/table_crc                |        476.43&nbsp;MiB/s   |                **475.45&nbsp;MiB/s** | **412.85&nbsp;MiB/s** |           **444.69&nbsp;MiB/s** |   213.55&nbsp;MiB/s   |       **213.32&nbsp;MiB/s** |   148.11&nbsp;MiB/s   |       **147.55&nbsp;MiB/s** |
| crc/small_table_crc          |        213.63&nbsp;MiB/s   |                  215.92&nbsp;MiB/s   |   186.30&nbsp;MiB/s   |             199.50&nbsp;MiB/s   |   95.077&nbsp;MiB/s   |         95.093&nbsp;MiB/s   |   80.944&nbsp;MiB/s   |         81.181&nbsp;MiB/s   |
| crc/barret_crc               |        147.86&nbsp;MiB/s   |                  50.603&nbsp;MiB/s   |   45.476&nbsp;MiB/s   |             27.352&nbsp;MiB/s   |   44.161&nbsp;MiB/s   |         16.026&nbsp;MiB/s   |   34.457&nbsp;MiB/s   |         6.8674&nbsp;MiB/s   |
| crc/word_barret_crc          |      **664.75&nbsp;MiB/s** |                  209.72&nbsp;MiB/s   |   208.67&nbsp;MiB/s   |             108.86&nbsp;MiB/s   | **213.85&nbsp;MiB/s** |         67.776&nbsp;MiB/s   | **362.81&nbsp;MiB/s** |         27.877&nbsp;MiB/s   |
| crc/reversed_barret_crc      |        127.57&nbsp;MiB/s   |                  50.016&nbsp;MiB/s   |   39.038&nbsp;MiB/s   |             24.993&nbsp;MiB/s   |   38.342&nbsp;MiB/s   |         12.993&nbsp;MiB/s   |   24.023&nbsp;MiB/s   |         5.6548&nbsp;MiB/s   |
| crc/word_reversed_barret_crc |        592.13&nbsp;MiB/s   |                  214.57&nbsp;MiB/s   |   194.16&nbsp;MiB/s   |             110.00&nbsp;MiB/s   |   191.32&nbsp;MiB/s   |         56.501&nbsp;MiB/s   |   166.23&nbsp;MiB/s   |         22.548&nbsp;MiB/s   |


