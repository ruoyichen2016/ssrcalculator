# Curated-core data review (37 cases)

Line-by-line review of the 37 curated cases against publicly documented
figures (dimensions, standard/normal displacement, block coefficient, speed and
shaft power).  This is a desk review from general reference knowledge, not a
citation-backed audit; values are confirmed "within the approximation" unless
noted.

Method

- Dimensions/displacement: checked against the documented standard/normal
  figures, and cross-checked with the engine's prismatic definition
  `modeled = L*B*T*Cb/35` (the engine ignores the stored displacement and uses
  this).  Ratio in [0.95, 1.60] is treated as consistent (stored displacement is
  standard/normal, the model is closer to full load).
- Speed/power: checked against documented design/trial figures.
- Result: 36 confirmed, 1 corrected (Sendai length).

Summary

- All 37 curated cases have a modeled/documented ratio within 0.95-1.59, so the
  curated core is dimensionally self-consistent.
- The 11 out-of-band cases are all in the 86-case approximate bulk; they are the
  best candidates for the next manual pass (see `data-consistency`).

| # | Case | Type | L ft | B ft | T ft | Disp t | Cb | kt | HP | modeled/doc | Status |
|---|------|------|------|------|------|--------|----|----|----|-------------|--------|
| 1 | HMS Dreadnought (1906) | battleship | 527 | 82 | 26 | 18200 | 0.61 | 21.0 | 23000 | 1.10 | confirmed |
| 2 | Bismarck (1940) | battleship | 823 | 118 | 30 | 45000 | 0.56 | 30.0 | 138000 | 1.05 | confirmed |
| 3 | Iowa (1943) | battleship | 860 | 108 | 36 | 48000 | 0.60 | 33.0 | 212000 | 1.19 | confirmed |
| 4 | Fletcher class DD (1942) | destroyer | 376 | 40 | 13 | 2100 | 0.48 | 36.5 | 60000 | 1.28 | confirmed |
| 5 | Yamato (1941) | battleship | 862 | 121 | 34 | 65000 | 0.61 | 27.0 | 150000 | 0.95 | confirmed |
| 6 | Kongo (refit 1936) | battleship | 728 | 101 | 32 | 36600 | 0.57 | 30.0 | 136000 | 1.05 | confirmed |
| 7 | Essex (1943) | carrier | 820 | 93 | 23 | 27100 | 0.59 | 33.0 | 150000 | 1.09 | confirmed |
| 8 | Atlanta class CL (1942) | cruiser | 541 | 53 | 17 | 6000 | 0.50 | 32.5 | 75000 | 1.16 | confirmed |
| 9 | Tribal class DD (1938) | destroyer | 377 | 36 | 11 | 1900 | 0.47 | 36.0 | 44000 | 1.06 | confirmed |
| 10 | South Dakota (1942) | battleship | 680 | 108 | 34 | 35000 | 0.60 | 27.5 | 130000 | 1.22 | confirmed |
| 11 | King George V (1940) | battleship | 745 | 103 | 34 | 36750 | 0.58 | 28.0 | 110000 | 1.18 | confirmed |
| 12 | Baltimore CA (1943) | cruiser | 673 | 71 | 22 | 13600 | 0.52 | 33.0 | 120000 | 1.15 | confirmed |
| 13 | Cleveland CL (1942) | cruiser | 608 | 66 | 20 | 10000 | 0.51 | 32.5 | 100000 | 1.17 | confirmed |
| 14 | Yorktown (1937) | carrier | 770 | 83 | 21 | 19800 | 0.55 | 32.5 | 120000 | 1.07 | confirmed |
| 15 | Illustrious (1940) | carrier | 740 | 95 | 24 | 23000 | 0.56 | 30.5 | 111000 | 1.17 | confirmed |
| 16 | North Carolina (1941) | battleship | 729 | 108 | 33 | 35000 | 0.61 | 28.0 | 121000 | 1.29 | confirmed |
| 17 | Vanguard (1946) | battleship | 814 | 108 | 30 | 44500 | 0.60 | 30.0 | 130000 | 1.02 | confirmed |
| 18 | Shokaku (1941) | carrier | 845 | 85 | 27 | 25675 | 0.58 | 34.2 | 160000 | 1.25 | confirmed |
| 19 | Hood (1920) | battleship | 860 | 104 | 30 | 42000 | 0.56 | 32.0 | 144000 | 1.02 | confirmed |
| 20 | Zara class CA (1931) | cruiser | 600 | 68 | 20 | 11500 | 0.52 | 32.0 | 95000 | 1.05 | confirmed |
| 21 | Mogami (1935) | cruiser | 650 | 66 | 18 | 11200 | 0.51 | 35.0 | 152000 | 1.00 | confirmed |
| 22 | Kagero class DD (1939) | destroyer | 388 | 38 | 12 | 2500 | 0.48 | 35.0 | 52000 | 0.97 | confirmed |
| 23 | Brooklyn CL (1937) | cruiser | 608 | 61 | 19 | 9700 | 0.53 | 32.5 | 100000 | 1.10 | confirmed |
| 24 | Dido CL (1940) | cruiser | 512 | 50 | 15 | 5600 | 0.51 | 32.2 | 62000 | 1.00 | confirmed |
| 25 | Somers DD (1937) | destroyer | 381 | 36 | 10 | 1840 | 0.47 | 36.0 | 52000 | 1.04 | confirmed |
| 26 | Benson DD (1942) | destroyer | 341 | 36 | 12 | 1620 | 0.47 | 37.5 | 50000 | 1.20 | confirmed |
| 27 | Independence CVL (1943) | carrier | 622 | 71 | 24 | 10662 | 0.56 | 31.6 | 100000 | 1.59 | confirmed |
| 28 | Bogue CVE (1943) | carrier | 496 | 69 | 24 | 9393 | 0.59 | 18.0 | 8500 | 1.47 | confirmed |
| 29 | Alaska CB (1944) | cruiser | 808 | 91 | 31 | 29779 | 0.55 | 33.0 | 150000 | 1.20 | confirmed |
| 30 | Gneisenau (1938) | battleship | 754 | 98 | 32 | 31500 | 0.56 | 32.0 | 160000 | 1.20 | confirmed |
| 31 | Porter class DD (1936) | destroyer | 381 | 36 | 10 | 1850 | 0.47 | 37.0 | 50000 | 1.04 | confirmed |
| 32 | Sims class DD (1939) | destroyer | 348 | 36 | 13 | 1570 | 0.47 | 37.0 | 51138 | 1.43 | confirmed |
| 33 | Mahan class DD (1936) | destroyer | 341 | 36 | 11 | 1500 | 0.47 | 37.0 | 46000 | 1.15 | confirmed |
| 34 | New Orleans class CA (1934) | cruiser | 588 | 62 | 19 | 9950 | 0.52 | 32.7 | 107000 | 1.05 | confirmed |
| 35 | Kuma class CL (1920) | cruiser | 532 | 47 | 16 | 5500 | 0.50 | 36.0 | 90000 | 1.01 | confirmed |
| 36 | Sendai class CL (1924) | cruiser | 500 | 46 | 16 | 5195 | 0.50 | 35.2 | 90000 | 1.00 | corrected (L 532->500) |
| 37 | Admiral Hipper class CA (1939) | cruiser | 665 | 70 | 24 | 14000 | 0.52 | 32.0 | 132000 | 1.16 | confirmed |

## Notes

- **Sendai class CL (1924)**: length corrected from 532 ft (Kuma class) to
  500 ft (152.4 m).  This also brought its modeled displacement in line with the
  documented 5,195 t.
- **Carrier beams**: Shokaku (and, in the approximate bulk, Wasp) used the
  flight-deck width; hull waterline beams are now used.
- **Displacement convention**: stored values are usually standard/normal, while
  the engine models a full-load-ish displacement from `L*B*T*Cb/35`; ratios
  around 1.2-1.4 are expected.  This is documented in `docs/HISTORICAL.md`.
- **Length convention**: some carriers use waterline length (Yorktown, Essex),
  others overall (Independence, Bogue).  Consistent within a case, but mixed
  across the set.

