# Iris dataset

`data/iris.csv` is a repository-local copy of `bezdekIris.data` from the UCI
Machine Learning Repository Iris dataset.

- Source: https://archive.ics.uci.edu/dataset/53/iris
- DOI: https://doi.org/10.24432/C56C76
- Creator: R. A. Fisher
- Citation: Fisher, R. (1936). Iris [Dataset]. UCI Machine Learning Repository.
- License: Creative Commons Attribution 4.0 International (CC BY 4.0)
- Retrieved: 2026-07-16

The vendored file differs only by the addition of the header
`sepal_length_cm,sepal_width_cm,petal_length_cm,petal_width_cm,species`.
Keeping the small input in the example makes the audit deterministic and
network-free.
