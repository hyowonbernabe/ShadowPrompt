# CSE 15 Data Mining — Prelim Exam Reference Notes

This file collects the lecture notes, quiz reviewers, and the methodology used in the prelim activities for CSE 15 (Data Mining). It is meant as a complete reference for exam review, so it favors completeness and precise formulas over brevity. Personal computed results from the original activity submissions are mostly left out; what is kept from the activities is the general method, the reasoning behind each step, and the checks that catch common mistakes.

Topics covered, in order:

1. Introduction to Data Mining (data quality, preprocessing, data reduction, transformation, discretization)
2. Feature Selection
3. Principal Component Analysis (PCA)
4. Logistic Regression
5. Similarity and Dissimilarity Measures
6. Hierarchical Clustering (AGNES and DIANA), density-based and grid-based clustering, clustering evaluation
7. Decision flowcharts for choosing the right similarity/clustering method

---

# 1. Introduction to Data Mining

## 1.1 Data quality dimensions

Data quality is judged along several dimensions:

- **Accuracy**: whether the data is correct or wrong.
- **Completeness**: whether values are recorded or missing/unavailable.
- **Consistency**: whether some values were updated while related ones were not (dangling references, etc.).
- **Timeliness**: whether the data reflects a timely update.
- **Believability**: how trustable the data is.
- **Interpretability**: how easily the data can be understood.

## 1.2 Data versus information

- **Data**: facts and statistics collected for reference or analysis; things assumed as facts that form the basis of reasoning.
- **Information**: what is conveyed by a particular arrangement of data; data as processed, stored, or transmitted.

## 1.3 Types of data and levels of measurement

**Quantitative vs qualitative data**

| | Quantitative Data | Qualitative Data |
|---|---|---|
| Association | Associated with numbers | Associated with details |
| When used | Data is numerical | Data can be segregated into well-defined groups |
| Analysis | Can be statistically analyzed | Can only be observed, not evaluated |
| Examples | Height, weight, time, price, temperature | Scents, appearance, beauty, colors, flavors |

**Levels of measurement** (scale of measurement, used to describe the information carried by values):

- **Nominal**: unordered categories (e.g. nationality, hair color).
- **Ordinal**: ordered categories with no fixed distance between them (e.g. level of service, educational level).
- **Interval**: ordered, equal distances, no true zero (e.g. IQ test).
- **Ratio**: ordered, equal distances, true zero (e.g. annual sales, voltage, crime rate, height).

**Types of dataset**: record, graph, ordered data, time series.

**The six Vs of big data**: Volume (amount of data), Variety (structured/semi-structured/unstructured), Velocity (speed of generation), Veracity (trustworthiness), Value (business value), Variability (how data can be used/formatted differently).

## 1.4 Taxonomy of data analytics

- **Descriptive Analytics**: summarizes/condenses data to extract patterns ("what happened").
- **Diagnostic Analytics**: diagnoses problems shown in the data ("why did it happen").
- **Predictive Analytics**: extracts models used for future predictions ("what will happen").
- **Prescriptive Analytics**: combines insight from the first three to support decisions ("how can we make it happen").
- **Cognitive Analytics**: uncovers hidden patterns, replicates human thought ("what is the extent of what can happen").

Complexity and value both increase moving from descriptive toward cognitive analytics; foresight/insight moves toward "wide sight" and "deep sight."

**Predictive analytics algorithms**:
- Supervised learning: classification, regression, time series analysis.
- Unsupervised learning: clustering, association analysis, sequential pattern analysis, text mining/social media sentiment analysis.

## 1.5 Major tasks in data preprocessing

- **Data cleaning**: fill in missing values, smooth noisy data, identify/remove outliers, resolve inconsistencies.
- **Data integration**: integration of multiple databases, data cubes, or files.
- **Data reduction**: dimensionality reduction, numerosity reduction, data compression.
- **Data transformation and discretization**: normalization, concept hierarchy generation.

### Data integration

- Combines data from multiple sources into a coherent store.
- **Schema integration**: e.g. matching `A.cust-id` with `B.cust-#`; integrating metadata from different sources.
- **Entity identification problem**: identifying that different records from different sources refer to the same real-world entity (e.g. "Bill Clinton" = "William Clinton").
- **Detecting and resolving data value conflicts**: the same real-world entity may have different attribute values across sources because of different representations or scales (metric vs. British units, etc.).

**Handling redundancy in data integration**:
- **Object identification**: the same attribute/object may have different names in different databases.
- **Derivable data**: one attribute may be derived from another (e.g. annual revenue derived from monthly figures).
- Redundant attributes can be detected by **correlation analysis** and **covariance analysis**.
- Careful integration reduces redundancy/inconsistency and improves mining speed and quality.

### Correlation analysis (numeric data)

Pearson's correlation coefficient:

$$r = \frac{n\sum XY - (\sum X)(\sum Y)}{\sqrt{[n\sum X^2 - (\sum X)^2][n\sum Y^2 - (\sum Y)^2]}}$$

- $r > 0$: A and B are positively correlated (A increases as B increases); higher $r$ means stronger correlation.
- $r = 0$: independent.
- $r < 0$: negatively correlated.

Worked example (streaming service, monthly fee X vs annual fee Y, n = 6): computing the formula gives $r = 1.00$, a perfect positive correlation. Since $|r| = 1.00$, one of the two attributes can be dropped during feature selection because it is fully redundant (annual fee is exactly monthly fee times a constant).

### Covariance (numeric data)

$$Cov(A,B) = E[(A-\bar A)(B-\bar B)] = \frac{\sum_{i=1}^n (a_i - \bar A)(b_i - \bar B)}{n}$$

Simplified computational form: $Cov(A,B) = E(A \cdot B) - \bar A \bar B$.

Correlation coefficient in terms of covariance: $r_{A,B} = \dfrac{Cov(A,B)}{\sigma_A \sigma_B}$.

- **Positive covariance**: if $Cov_{A,B} > 0$, A and B tend to be larger than their expected values together.
- **Negative covariance**: if $Cov_{A,B} < 0$, when A is above its expected value B tends to be below its expected value.
- **Independence**: $Cov_{A,B} = 0$ implies nothing is necessarily concluded about independence in the reverse direction — some independent-looking pairs can have zero covariance without being independent; only under extra assumptions (e.g. multivariate normal data) does zero covariance imply independence.

### Correlation analysis (nominal data): chi-square test

$$\chi^2 = \sum \frac{(Observed - Expected)^2}{Expected}$$

- The larger the $\chi^2$ value, the more likely the variables are related.
- The cells contributing most to $\chi^2$ are those whose actual count differs most from the expected count.
- Expected count formula: $E = \dfrac{(\text{Row Total})(\text{Column Total})}{\text{Grand Total}}$.
- Degrees of freedom for a contingency table: $(r-1)(c-1)$.
- **Correlation does not imply causality** — e.g. the number of hospitals and the number of car thefts in a city can be correlated because both are driven by a third variable, population.

Worked example: subscription plan (Basic/Premium) vs streaming quality (SD/HD), 300 users. Expected counts computed from row/column totals, then $\chi^2 = 85.72$. With 1 degree of freedom and $\alpha = 0.05$, the critical value is 3.841. Since $85.72 > 3.841$, reject $H_0$: the two attributes are associated (Basic subscribers mostly use SD, Premium mostly use HD). Since the two attributes convey similar information, one may be considered redundant with the other during feature selection.

## 1.6 Data reduction strategies

Data reduction produces a smaller representation of a dataset that still yields the same (or nearly the same) analytical results. Needed because complex analysis on huge datasets can take too long.

**Dimensionality reduction** — remove unimportant attributes:
- Wavelet transforms
- Principal Components Analysis (PCA) — see Section 3
- Feature subset selection, feature creation — see Section 2

**Numerosity reduction** (sometimes just called "data reduction"):
- Parametric methods (e.g. regression, log-linear models): assume the data fits a model, store only the parameters (plus possible outliers), discard the rest.
- Non-parametric methods: histograms, clustering, sampling.

**Data compression**:
- String compression: typically lossless, limited manipulation without expansion.
- Audio/video compression: typically lossy, with progressive refinement.
- Time sequence compression: signals are typically short and vary slowly.
- Dimensionality and numerosity reduction can themselves be viewed as forms of data compression.

### Histogram analysis

Divide data into buckets and store the average/sum per bucket.
- **Equal-width**: equal bucket range.
- **Equal-frequency (equal-depth)**: each bucket holds about the same number of samples.

### Clustering (as a reduction technique)

Partition the dataset into clusters based on similarity, and store only cluster representations (e.g. centroid and diameter). Effective if the data actually clusters well, less effective if data is "smeared." Can be hierarchical and stored in multi-dimensional index tree structures.

### Sampling

Obtaining a small sample $s$ to represent the whole dataset $N$, letting a mining algorithm run in sub-linear time relative to the full data size. Key principle: choose a **representative** subset.

- **Simple random sampling**: equal probability of selecting any item.
  - **Without replacement**: once selected, an object is removed from the population.
  - **With replacement**: a selected object stays in the population and can be selected again.
- **Stratified sampling**: partition the dataset, draw samples from each partition proportionally; used with skewed data.
- Simple random sampling performs poorly under skew.
- Sampling does not necessarily reduce database I/O (still page-at-a-time).

### Data cube aggregation

- The base cuboid is the lowest level of a data cube, holding aggregated data for an individual entity of interest.
- Multiple levels of aggregation further reduce data size.
- Use the smallest representation sufficient to answer the task; answer aggregate queries from the cube when possible.

## 1.7 Data transformation

A function mapping the entire set of values of an attribute to a new set of replacement values, so each old value corresponds to exactly one new value.

Methods:
- **Smoothing**: remove noise from data.
- **Attribute/feature construction**: build new attributes from existing ones.
- **Aggregation**: summarization, data cube construction.
- **Normalization**: scale to fall within a smaller, specified range.
- **Discretization**: concept hierarchy climbing.

### Normalization

**Min-max normalization** to $[\text{new\_min}_A, \text{new\_max}_A]$:

$$v' = \frac{v - min_A}{max_A - min_A}(new\_max_A - new\_min_A) + new\_min_A$$

Example: income range \$12,000–\$98,000 normalized to $[0.0, 1.0]$. \$73,000 maps to $\frac{73{,}000-12{,}000}{98{,}000-12{,}000}(1.0-0)+0 = 0.716$.

**Z-score normalization** ($\mu$: mean, $\sigma$: standard deviation):

$$v' = \frac{v-\mu_A}{\sigma_A}$$

Example: $\mu=54{,}000$, $\sigma=16{,}000$. For $v=73{,}600$: $v' = \frac{73{,}600-54{,}000}{16{,}000}=1.225$.

**Normalization by decimal scaling**:

$$v' = \frac{v}{10^j}, \quad \text{where } j \text{ is the smallest integer such that } \max(|v'|) < 1$$

Example: values range from -986 to 917; the maximum absolute value is 986, so divide every value by 1000 (j = 3). -986 → -0.986, 917 → 0.917.

### Discretization

Three types of attributes for discretization purposes:
- **Nominal**: unordered set of values (e.g. color, profession).
- **Ordinal**: ordered set of values (e.g. military/academic rank).
- **Numeric**: real numbers.

Discretization divides the range of a continuous attribute into intervals, and interval labels replace the actual values, reducing data size. It can be:
- **Supervised** vs **unsupervised**.
- **Split** (top-down) vs **merge** (bottom-up).
- Applied recursively on an attribute; often prepares data for classification.

**Data discretization methods** (all can be applied recursively):
- **Binning**: top-down split, unsupervised.
- **Histogram analysis**: top-down split, unsupervised.
- **Clustering analysis**: unsupervised, top-down split or bottom-up merge.
- **Decision-tree analysis**: supervised, top-down split.
- **Correlation analysis** (e.g. $\chi^2$): unsupervised, bottom-up merge.

**Equal-width (distance) binning**: width $w = (B - A)/N$ where $A$, $B$ are the lowest/highest attribute values and $N$ is the number of bins.

Example: dataset 10, 15, 18, 20, 31, 34, 41, 46, 51, 53, 54; $N=4$. Width $= (54-10)/4 = 11$.
- Bin 1: [10, 20] → {10, 15, 18, 20}
- Bin 2: [21, 31] → {31}
- Bin 3: [32, 42] → {34, 41}
- Bin 4: [43, 54] → {46, 51, 53, 54}

Equal-width is straightforward but outliers can dominate presentation; it handles skewed data poorly.

**Equal-depth (frequency) binning**: divides the range into $N$ intervals each holding roughly the same number of samples. Frequency = total data points / number of bins. Good for skewed/non-uniform data since it balances the distribution across bins; harder to apply cleanly to categorical attributes.

Example: dataset 10,15,18,20,31,34,41,46,51,53,54,60; 3 bins; frequency = 12/3 = 4.
- Bin 1: 10, 15, 18, 20
- Bin 2: 31, 34, 41, 46
- Bin 3: 51, 53, 54, 60

**Data smoothing** (applied after binning, to simplify the values inside each bin):
- **Smooth by bin mean**: replace every value in a bin with the bin's mean.
- **Smooth by bin median**: replace every value with the bin's median.
- **Smooth by bin boundaries**: replace each value with whichever bin boundary (min or max) it is closer to.

Example (bin mean): Bin 1 = {4,7,13,16}, mean 10 → all become 10. This reduces the influence of outliers/extreme values within each bin.

Example (bin boundaries): Bin 1 = {4,7,13,16}, min 4 max 16. Value 7 is closer to 4 so becomes 4; value 13 is closer to 16 so becomes 16. Final bin: [4, 4, 16, 16].

### Concept hierarchy generation

- A concept hierarchy organizes attribute values hierarchically, usually per dimension in a data warehouse.
- Facilitates drilling and rolling to view data at multiple granularities.
- Formed by recursively replacing low-level concepts (numeric age values) with higher-level concepts (youth, adult, senior).
- Can be specified explicitly by domain experts, or generated automatically.
- For numeric data, discretization methods (above) are used to build the hierarchy.

**For nominal data**:
- Explicit partial/total ordering of attributes at the schema level (e.g. `street < city < state < country`).
- Explicit grouping of values into a hierarchy (e.g. `{Urbana, Champaign, Chicago} < Illinois`).
- Specifying only a partial set of attributes (e.g. only `street < city`).
- Automatic generation based on the number of distinct values per attribute: the attribute with the most distinct values is placed at the lowest level (exceptions exist, e.g. weekday/month/quarter/year do not follow this rule simply).

## 1.8 Summary

Data quality (accuracy, completeness, consistency, timeliness, believability, interpretability) → data cleaning (missing/noisy values, outliers) → data integration (entity identification, removing redundancy, detecting inconsistency) → data reduction (dimensionality reduction, numerosity reduction, data compression) → data transformation/discretization (normalization, concept hierarchy generation).

---

# 2. Feature Selection

## 2.1 Definition and purpose

**Feature selection** is the process of choosing only the most useful input features for a machine learning model.

Benefits:
- Removes irrelevant and redundant features.
- Improves model performance and reduces overfitting.
- Reduces noise.
- Speeds up model training.
- Makes models simpler and easier to interpret.

## 2.2 Types of feature selection methods

### Supervised feature selection methods

Identify which input variables most directly affect the target variable. Correlation is the primary criterion for assessing importance. Subtypes: **filter methods**, **wrapper methods**, **embedded methods**, **hybrid methods**.

**Filter methods** (available in scikit-learn):

- **Information gain**: measures how important a feature's presence/absence is for determining the target, via the degree of entropy reduction.
- **Mutual information**: assesses dependence between variables by measuring the information obtained about one through the other.
- **Chi-square test**: assesses the relationship between two categorical variables by comparing observed to expected values (see Section 1.5).
- **Fisher's score**: uses derivatives to calculate each feature's relative importance for classification; a higher score means greater influence.
- **Pearson's correlation coefficient**: quantifies the linear relationship between two continuous variables, ranging from -1 to 1.
- **Variance threshold**: removes features whose variance falls under a minimum, since higher-variance features are assumed to carry more information. Related: mean absolute difference (MAD).
- **Missing value ratio**: percentage of instances for which a feature is missing/null; if too many are missing, the feature is unlikely to be useful.
- **Dispersion ratio**: ratio of variance to mean value for a feature; higher dispersion indicates more information.
- **ANOVA (analysis of variance)**: determines whether different feature values affect the value of the target variable.

**Wrapper methods** — train the model on various feature subsets, adding/removing features and testing at each iteration to find the subset with optimal performance:

- **Forward selection**: starts with an empty set, gradually adds features until optimal.
- **Backward selection**: starts with all features, iteratively removes the least important.
- **Exhaustive feature selection**: tests every possible combination to find the overall best by a performance metric.
- **Recursive feature elimination (RFE)**: a type of backward selection that eliminates/adds features each iteration based on relative importance.
- **Recursive feature elimination with cross-validation**: RFE variant that uses cross-validation (testing on unseen data) to pick the best-performing feature set.

**Embedded methods** — feature selection is folded into model training; the model detects underperforming features during training and discards them in later iterations:

- **LASSO regression (L1 regression)**: penalizes high-value correlated coefficients, moving them toward 0; coefficients that hit 0 are removed.
- **Random forest importance**: builds many decision trees on random subsets of data and features; a feature's importance is based on how well trees using it split the data.
- **Gradient boosting**: adds predictors sequentially, each correcting the errors of the previous one.

### Unsupervised feature selection methods

- **Principal component analysis (PCA)**: reduces dimensionality by transforming correlated variables into a smaller set of components that retain most of the information; counters the curse of dimensionality and reduces overfitting. See Section 3.
- **Independent component analysis (ICA)**: separates multivariate data into statistically independent components.
- **Autoencoders**: neural-network based dimensionality reduction.

## 2.3 Attribute subset selection and heuristic search

**Attribute subset selection** is another way to reduce dimensionality (besides PCA):
- **Redundant attributes**: duplicate information contained in one or more other attributes (e.g. purchase price and sales tax paid).
- **Irrelevant attributes**: contain no information useful for the mining task (e.g. student ID for predicting GPA).

There are $2^d$ possible attribute combinations for $d$ attributes, so heuristic methods are used:
- **Best single attribute** under the attribute independence assumption: chosen by significance tests.
- **Best step-wise feature selection**: pick the best single attribute first, then the next best conditional on the first, and so on.
- **Step-wise attribute elimination**: repeatedly eliminate the worst attribute.
- **Best combined attribute selection and elimination**.
- **Optimal branch and bound**: uses attribute elimination and backtracking.

**Attribute creation (feature generation)**: create new attributes that capture important information more effectively than the originals.
- Attribute extraction (domain-specific).
- Attribute construction (combining features, data discretization).

## 2.4 Applied methodology (from Prelim Activity 1)

The activity used the mobile-phone `train.csv` dataset (20 features, target `price_range`) to illustrate the three basic filter/embedded techniques, then applied the same three techniques to `glass.csv` for an actual model comparison. The general workflow and reasoning (not the specific numeric results) is summarized below, since it is the reusable exam-relevant method:

**Univariate selection** — `SelectKBest(score_func=chi2, k=...)`. Chi-square requires non-negative input, so features must first be scaled into a non-negative range (e.g. `MinMaxScaler`, not `StandardScaler` which produces negative values). Each feature is scored independently against the target; the top-k by score are kept. Weakness: since each feature is tested alone, it cannot detect a feature that is only useful in combination with another (e.g. a feature that is usually zero but a strong signal on the rare rows where it isn't may rank artificially high, while a feature useful only after another split has been made may rank artificially low).

**Feature importance** — trains a tree-based ensemble (`ExtraTreesClassifier` or `RandomForestClassifier`) on all features and reads off `.feature_importances_`, which are shares that sum to 1. Because the model sees all features together, it can credit a feature for value it adds *after* another feature has already been used to split the data — the exact blind spot of univariate selection. `random_state` must be set since these ensembles are otherwise stochastic and rankings can shift between runs (though usually the top few stay stable).

**Correlation method** — compute the full correlation matrix (heatmap), then rank features by absolute correlation with the target. This only detects straight-line relationships, so a feature that matters through a non-linear relationship can rank artificially low even though a tree-based method would rank it high. The correlation matrix is also used to spot redundant *predictor pairs* (features highly correlated with each other, not just with the target) — keeping both wastes model capacity on the same information.

**General procedure for comparing feature sets**:
1. Clean the data first: drop pure identifier columns (e.g. row ID) before running any feature selection, since an identifier has variance but no real relationship to the target and would otherwise get ranked. Check and drop exact duplicate rows (ignoring the ID column, since duplicated measurements with different IDs would still bias results and could straddle the train/test split).
2. Check class balance; with a very uneven target, stratify the train/test split so rare classes are represented in both sets, and prefer macro-averaged precision/recall/F1 over plain accuracy, since accuracy alone rewards a model that only predicts the majority class(es).
3. Split data (e.g. 80/20) with a fixed `random_state`/seed for reproducibility, and use stratified k-fold cross-validation on the training set for model selection.
4. Run each feature-selection technique on the **training data only** (fit any scaler on train, then transform test) to avoid leaking test information into the selection process.
5. Train the same classifier once per feature set (all features, and each top-k feature set) using the same seed, and compare cross-validation accuracy (mean and std) plus held-out test metrics (accuracy, macro precision/recall/F1).
6. The three methods often disagree on which features matter, because they measure different things (single-feature relationship vs. relationship in combination vs. straight-line relationship). Features appearing in all three top-k lists are the safest picks. A tree-based / combination-aware method (feature importance) is often more reliable than the two single-feature methods (univariate selection, correlation) when features interact or have non-linear effects, and this can be confirmed by checking which feature set actually gives the best held-out performance.
7. Reducing to a well-chosen feature subset does not have to cost accuracy — it can even improve held-out performance versus using all features, if the removed features were mostly redundant or noisy relative to the kept ones.

---

# 3. Principal Component Analysis (PCA)

## 3.1 What PCA does

PCA reduces the number of dimensions in a large dataset to a smaller number of **principal components** that retain most of the original information. It aims to display the relative positions of data points in fewer dimensions while retaining as much information as possible, and to explore relationships between variables. PCA also minimizes/eliminates issues like multicollinearity and overfitting. It **works on numeric data only**.

### Curse of dimensionality

As dimensionality increases, data becomes increasingly sparse; density and distance between points (critical for clustering and outlier analysis) become less meaningful; the number of possible subspace combinations grows exponentially. Dimensionality reduction avoids this curse, eliminates irrelevant features, reduces noise, reduces time/space requirements, and allows easier visualization.

## 3.2 Principal components

- Principal components are **linear combinations of the original variables** that have maximum variance compared to other linear combinations; they capture as much information (variance) from the dataset as possible.
- **PC1** is the direction in space along which the data has the highest variance — the line best representing the shape of the projected points. The larger the variability captured by the first component, the more information retained.
- **PC2** accounts for the next-highest variance and must be **uncorrelated with (orthogonal/perpendicular to) PC1**. Correlation between PC1 and PC2 equals zero.
- Components are sorted by decreasing "significance" (strength); the data size can be reduced by dropping weak components (low variance) while still reconstructing a good approximation of the original data using the strongest components.
- PCA finds a projection capturing the largest amount of variation in data by finding the **eigenvectors of the covariance matrix**; these eigenvectors define the new space.

### Steps

Given $N$ data vectors from $n$ dimensions, find $k \le n$ orthogonal vectors (principal components) that best represent the data:
1. **Normalize input data** so each attribute falls within the same range (standardize).
2. Compute $k$ orthonormal (unit) vectors — the principal components.
3. Each input vector becomes a linear combination of the $k$ principal component vectors.
4. Components are sorted in decreasing order of significance/strength (eigenvalue).
5. Reduce data size by eliminating weak (low-variance) components; the strongest components can reconstruct a good approximation of the original data.

## 3.3 Eigenvalues, explained variance, loadings

- **Eigenvalue** (a component's "Total" in older software output): the amount of variance in the original variables accounted for by that component.
- **% of Variance**: the ratio of the variance accounted for by a component to the total variance across all variables, expressed as a percentage — this is the same as the **explained variance ratio**.
- **Cumulative %**: running total of % of variance across the first $n$ components.
- **Explained Variance**: the variance explained by each principal component (eigenvector). These eigenvectors represent the components containing most of the information (variance) originally spread across the features.
- A sharp drop from one eigenvalue to the next (a large successive difference) can indicate how many eigenvalues/components to keep.

**Loadings vs eigenvectors**: the raw eigenvector entries are unit-length *weights* describing how each original variable combines to form a component; they are not directly comparable to correlations. The **loading** of a variable on a component is the eigenvector entry multiplied by the square root of that component's eigenvalue:

$$\text{loading}_{var,PC} = \text{eigenvector}_{var,PC} \times \sqrt{\text{eigenvalue}_{PC}}$$

Loadings can be read as the correlation between the original variable and the component, on the familiar -1 to 1 scale. A common guideline is that a loading of **0.70 or above** (in absolute value) marks a variable as a strong contributor to that component. Every original variable participates in every component (with some weight, however small); PCA never literally drops a variable, it only drops whole components when you choose to keep fewer than all of them. This is the key conceptual difference from feature selection, which drops entire original columns and keeps the survivors in their original, interpretable units.

Signs of loadings/eigenvectors are **arbitrary** — PCA could flip a whole component's sign and produce an equally valid solution, so only relative signs *within* a component are meaningful (e.g. two variables loading with opposite signs on the same component move in opposite directions along it).

## 3.4 How many components to keep

Three common, complementary criteria:

1. **Kaiser criterion**: keep components with eigenvalue > 1 (a component should explain at least as much variance as one original standardized variable).
2. **Cumulative explained variance threshold**: keep the smallest number of components whose cumulative explained variance reaches a chosen threshold, commonly **80%** or **90%**.
3. **Scree plot elbow**: plot eigenvalue (y) vs. component number (x); look for the point where the curve stops falling steeply and flattens out ("the elbow"). Components before the elbow are kept.

These criteria can disagree (e.g. a component might have an eigenvalue just under 1, failing Kaiser, but still be needed to represent an entire original variable that loads nowhere else strongly, and the scree elbow and 90% threshold might agree on keeping it). When they disagree, look at what each borderline component actually represents (via its loadings) before deciding — a component that is the *only* place a given original variable loads strongly is a good reason to keep it even if it narrowly fails one numeric rule.

Also useful: examine successive **differences between eigenvalues** — a sharp drop from one eigenvalue to the next is itself evidence for cutting there.

## 3.5 Standardization before PCA

PCA finds directions of maximum **variance**, and variance depends entirely on the units a variable happens to be measured in. If variables have wildly different scales/variances (e.g. one variable's variance is in the hundreds of thousands, another's is a tiny fraction), running PCA on raw (unstandardized) data lets the large-variance variable dominate PC1 almost completely, while small-variance variables become numerically invisible — this reflects measurement units, not real relationships. **Standardizing** (subtracting the mean, dividing by the standard deviation, i.e. z-score normalization) gives every variable a mean of 0 and standard deviation of 1 before PCA, which makes PCA operate on the **correlation matrix** rather than the raw covariance matrix, and gives every variable an equal starting claim on the components. Standardizing does not change the correlation matrix (correlation is already scale-free) but is essential once PCA computes variance/covariance from the data.

## 3.6 Biplots and visualization

A **biplot** overlays the observation scores (as points, typically on PC1 vs PC2) and the variable loading vectors (as arrows from the origin) on the same two axes. Reading a biplot:
- Two arrows pointing in nearly the same direction represent variables that are strongly positively correlated with each other.
- Two arrows pointing in opposite directions represent variables that are strongly negatively correlated.
- Two arrows at roughly 90° represent variables with little linear relationship.
- The angle between two arrows approximates their correlation.
- A single round, centered cloud of points with no visible clusters or trend indicates the observations vary smoothly with no natural grouping, and confirms the two plotted components are uncorrelated (as PCA guarantees by construction).

A **scree plot** plots eigenvalue against component number (optionally with a horizontal line at eigenvalue = 1 for the Kaiser criterion). A **cumulative explained variance plot** plots cumulative % variance against number of components, often with horizontal reference lines at 80% and 90%.

## 3.7 Limitations of PCA

- A component is a weighted mixture of the original variables, not something an instrument measures directly — it typically has no natural unit and cannot be read off a sensor.
- Component signs are arbitrary (see above).
- PCA only detects **linear** structure; a variable related to others by division or another non-linear relationship (e.g. resistance = voltage / current, an exact relationship from Ohm's law) cannot be folded into another component the way a linear relationship can, and may need a whole extra component to represent.
- The 0.70 loading cutoff is a guideline, not a hard rule; naming a component when no loading clears the guideline is a judgment call.
- Standardizing throws away the original units, so components cannot be traced back to physical magnitudes directly.
- PCA maximizes variance, not domain importance — a variable that matters a great deal to the domain but happens to vary very little in the sample gets pushed into a small, easily-dropped component.
- Running PCA without standardizing (when variables have very different scales) would produce a completely different, misleading result dominated by whichever variable happens to have the largest raw variance.

## 3.8 Applied methodology notes (from Prelim Activity 2)

General method used across three worked datasets of increasing size, illustrating when PCA is/is not worth doing:

- Load data with a fallback (`filename` if present in the notebook folder, else `data/filename`) so the same notebook works locally and on Colab.
- Drop non-numeric / identifier columns before PCA (e.g. email, address, a row-ID column) — an identifier has variance but no real relationship to anything and would otherwise be treated as signal.
- Check `.describe()` for scale differences (variance spanning many orders of magnitude across columns) before deciding standardization is necessary — it essentially always is, unless columns already share the same scale and similar spread.
- Fit `PCA()` with no `n_components` argument first (keeps all components) so the eigenvalues/cumulative variance can be examined before deciding how many to keep, rather than guessing a number up front.
- **How much PCA "pays off" depends entirely on how correlated the original variables are.** A block of strongly correlated variables (e.g. inter-related physical measurements, or neighboring pixels in an image) compresses into very few components holding most of the variance (steep/curved cumulative-variance curve). A set of largely uncorrelated variables (e.g. unrelated survey questions) compresses poorly — the cumulative-variance curve is close to a straight diagonal line, and cutting the number of variables down substantially costs a large fraction of the information; in that case PCA may not be a useful simplification even though it can still be computed.
- `IncrementalPCA` (fits in batches instead of holding the whole matrix in memory) is used for very wide datasets (e.g. hundreds of pixel columns across many rows) where ordinary `PCA` would be memory-heavy.
- Practical check: verify no missing values and no duplicate rows before PCA (same reasoning as feature selection — duplicates would double-count and could leak across any downstream train/test split).
- Two original variables tied by an exact non-linear relationship (like resistance = voltage/current, or period = 1/frequency) will show up as a very strong (but not perfect) linear correlation between them (e.g. -0.976 instead of exactly -1, because the relationship is a curve and Pearson correlation only measures straight lines) — and one of the two may end up needing its own separate component precisely because PCA can't fully absorb non-linear structure into components built from the other variables.
- When multiple methods disagree by a small margin (e.g. Kaiser says 3 components, cumulative variance/scree elbow say 4), it is reasonable to check which original variables would be poorly represented by dropping the marginal component — if a whole original variable only loads strongly on that one component, keeping it preserves more real information than blindly following the majority rule.

---

# 4. Logistic Regression

## 4.1 What it is

Logistic regression models the relationship between one or more predictor variables and a response variable where the **output is categorical**. It is used in classification tasks to predict the probability that an observation belongs to a class.

Typical examples: predicting whether a customer buys a product; predicting whether a team wins or loses a match.

## 4.2 Linear vs logistic regression

| | Linear regression | Logistic regression |
|---|---|---|
| Predictor variables | Continuous numeric/categorical | Continuous numeric/categorical |
| Output variable | Continuous numeric | Categorical |
| Relationship | Linear | Linear (with transformation) |
| Outcome assumption | Normally distributed | Not normally distributed |
| Regression line | Straight line | S-shaped curve |
| Output | Continuous values | Probability values |
| Error metric | Mean squared error | Logarithmic loss |

Linear regression: $y = a + bx$, where $y$ (continuous, $-\infty$ to $+\infty$) is fit by minimizing the sum of squared errors. Logistic regression's output $y$ can only take discrete values (0 or 1), so the straight-line equation cannot directly predict $y$; instead it predicts a probability, which is bounded in $[0,1]$, using the **logistic (sigmoid) function**.

## 4.3 Types of logistic regression

1. **Binary logistic regression**: response has exactly two outcomes (0/1) — e.g. diabetes yes/no.
2. **Multinomial logistic regression**: response has more than two categories, still nominal (unordered) — e.g. car color red/blue/green.
3. **Ordinal logistic regression**: response is ordinal (has a natural ordering) — e.g. education level.

## 4.4 Probability and odds

**Probability** is the fraction of times an event is expected in many trials; always between 0 and 1.

**Odds of success** for a group = ratio of probability of success to probability of failure:

$$\text{Odds} = \frac{P}{1-P}$$

- Odds > 1: success is more likely than failure for that group.
- Odds < 1: failure is more likely.
- Odds range from 0 to infinity.

**Odds ratio** compares odds of success between two groups:

$$\text{Odds ratio} = \frac{\text{Odds of success in Group 1}}{\text{Odds of success in Group 2}} = \frac{P_1/(1-P_1)}{P_2/(1-P_2)}$$

- Odds ratio = 1: no association between the grouping variable and the outcome.
- Odds ratio > 1: the event is more likely in Group 1.
- Odds ratio < 1: the event is more likely in Group 2.

## 4.5 Deriving the logistic function

Start from wanting to model $P$ (a probability, range 0 to 1) as a linear function of $X$ (range $-\infty$ to $+\infty$) — direct equality $P = a + bX$ doesn't work because the ranges don't match.

1. Replace $P$ with the **odds**, $\dfrac{P}{1-P}$, which ranges from 0 to $+\infty$: $\dfrac{P}{1-P} = a + bX$. Ranges still don't match (0 to $\infty$ vs. $-\infty$ to $\infty$).
2. Take the natural log of the odds (the **logit**): $\log\left(\dfrac{P}{1-P}\right) = a + bX$. Now both sides range over all real numbers.
3. Solving for $P$ gives the logistic (sigmoid) function:

$$P = \frac{e^{a+bX}}{1+e^{a+bX}} = \frac{1}{1+e^{-(a+bX)}}$$

- If $a + bX$ is very negative, $P \to 0$.
- If $a + bX$ is very large (positive), $P \to 1$.
- If $a + bX = 0$, $P = 0.5$.

**Inverse logit**: $\text{logit}(p) = \ln\left(\dfrac{p}{1-p}\right)$, and $\text{logit}^{-1}(\alpha) = \dfrac{1}{1-e^{-\alpha}} = \dfrac{e^\alpha}{1-e^\alpha}$ (equivalently the sigmoid form above), turning a linear combination of variables/coefficients back into a probability (the "event occurs" probability). This inverse-logit / sigmoid function is what produces the S-shaped curve.

**For multiple predictors** (multiple logistic regression):

$$P = \frac{e^{a+b\cdot X_i'}}{1+e^{a+b\cdot X_i'}} = \frac{1}{1+e^{-(a+b\cdot X_i')}}$$

**Decision boundary**: the boundary separating predicted classes — a line in 2D, a hyperplane in higher dimensions — determined by the model's coefficients/weights; it is the set of points where the model is equally likely to predict either class (i.e. where predicted probability = the chosen threshold, typically 0.5).

## 4.6 Worked numeric derivation pattern

Given fitted coefficients (e.g. $\beta_0=-9.346$, $\beta_1=0.014634$ for a credit-score model predicting loan approval):

- **Predict probability for a given X**: plug into $\hat p = \dfrac{e^{\beta_0+\beta_1 X}}{1+e^{\beta_0+\beta_1 X}}$.
- **Find X for a target odds/probability** (e.g. what score gives 3:1 odds, i.e. $P=0.75$): set $\ln\left(\dfrac{P}{1-P}\right) = \beta_0+\beta_1 X$, solve for $X$. E.g. $\ln(3) = -9.346+0.014634X \Rightarrow X \approx 713.7$.

## 4.7 Interpreting coefficients and odds ratios

For a fitted model such as $\ln\left(\dfrac{p}{1-p}\right) = -2 + 0.05\times\text{Income} + 1.2\times\text{CreditScore}$:

- **Intercept**: the log-odds when all predictors are zero (often not practically meaningful if zero isn't a realistic value for a predictor).
- **Coefficient interpretation**: a one-unit increase in a predictor changes the log-odds by that coefficient's value. Converting to an odds ratio via $e^{\beta}$ gives a multiplicative effect on the **odds** (not probability) per one-unit increase.
  - E.g. $\beta_{\text{Income}} = 0.05 \Rightarrow e^{0.05}\approx 1.05$: each additional unit of income increases the odds of approval by about 5%.
  - E.g. $\beta_{\text{CreditScore}} = 1.2 \Rightarrow e^{1.2}\approx 3.32$: a one-unit increase in credit score roughly triples the odds of approval.
- Odds ratio $>1$: increasing that predictor increases the odds of the outcome. Odds ratio $<1$: increasing that predictor decreases the odds. Odds ratio $=1$: no effect.

## 4.8 Evaluating logistic regression models

**Confusion matrix** (Actual rows, Predicted columns):

| | Predicted Positive (P') | Predicted Negative (N') |
|---|---|---|
| **Actual Positive (P)** | True Positive (TP) | False Negative (FN) |
| **Actual Negative (N)** | False Positive (FP) | True Negative (TN) |

**True/False positive rate**:
$$TPR = \frac{TP}{TP+FN} \quad \text{(higher is better)} \qquad FPR = \frac{FP}{FP+TN} \quad \text{(lower is better)}$$

**Precision, recall, F1**:
$$PRE = \frac{TP}{TP+FP} \quad \text{(higher is better)} \qquad REC = \frac{TP}{TP+FN} = TPR \quad \text{(higher is better)}$$
$$F1 = 2 \times \frac{PRE \times REC}{PRE + REC}$$

**ROC curve** (Receiver Operating Characteristic): plots TPR (y) vs. FPR (x) across all possible classification thresholds.
- A perfect classifier's ROC curve goes from (0,0) straight up to (0,1), then across to (1,1).
- A diagonal line (from (0,0) to (1,1)) represents random guessing.
- A useful classifier's curve falls between the diagonal and the perfect curve.
- Comparing classifiers: at a given tolerated FPR (e.g. no more than 10%), pick whichever classifier's curve reaches the highest TPR at that FPR; the best choice can differ depending on which FPR you're willing to tolerate.

**AUC** (Area Under the ROC Curve): a perfect classifier has AUC = 1.0; random guessing gives AUC = 0.5.

Implementations in Python: `statsmodels.formula.api.Logit()`, `sklearn.linear_model.LogisticRegression()`, `statsmodels.api.Logit()`, `statsmodels.api.GLM()`.

## 4.9 Applied methodology (from Prelim Activity 3): preprocessing checks for logistic regression

The activity required a systematic set of pre-modeling checks beyond ordinary cleaning; these are generally applicable whenever building a logistic regression model:

**Class imbalance and SMOTE**. Check the target's class balance (`value_counts`). A skewed target (e.g. roughly 70/30) biases a naive model toward the majority class — a model that always predicts the majority class can still score well on raw accuracy while catching none of the minority class. **SMOTE** (Synthetic Minority Oversampling Technique) generates synthetic minority-class examples to rebalance the training data. Critically, when using cross-validation, SMOTE must be applied **inside each fold** (e.g. via an imbalanced-learn `Pipeline` combining `SMOTE` + the classifier, rather than oversampling the whole dataset once before splitting) — otherwise synthetic points derived from a validation-fold observation can leak into that same fold's training data, artificially inflating the cross-validation score. Report a minority-class-specific metric (e.g. recall on the minority/"positive" class of interest) rather than trusting overall accuracy alone.

**Linearity of independent variables and log-odds (Box-Tidwell test)**. Logistic regression assumes a linear relationship between each continuous predictor and the **log-odds** (logit) of the outcome, not the outcome itself. The Box-Tidwell approach adds an interaction term for each continuous predictor $X$ of the form $X \times \ln(X)$ (shifting $X$ to be positive first, e.g. $X - \min(X) + 1$, since $\ln$ is undefined for zero/negative values) into the model, then checks whether that interaction term's p-value is significant. A significant p-value on the $X\times\ln X$ term indicates the true relationship between $X$ and the log-odds is not linear (it "bends"), and a transformation of $X$ may be needed. Non-significant p-values across all such terms mean the predictors can be used in their raw form.

**Strongly influential outliers (Cook's distance)**. Cook's distance measures how much the entire fitted model would change if a single observation were deleted — it identifies observations that are dragging the fit, not merely observations with unusual-looking values. A common cutoff is $4/n$ (n = number of observations); values exceeding 1 are a stronger signal of a genuinely problematic point. Fit the full model once (all observations, all predictors, categorical variables one-hot encoded) to compute per-observation Cook's distances, then check how many exceed the cutoff and what the maximum value is. Routinely, a moderate number of points will exceed the $4/n$ cutoff on almost any dataset (it is a deliberately sensitive rule), so the more important number is the maximum value relative to 1: if nothing approaches 1, no observation is meaningfully distorting the fit, and there is no strong reason to delete any real observations from the data.

**Absence of multicollinearity (correlation matrix + VIF)**. Two complementary checks:
- The **correlation matrix** among all numeric predictors (and with the target), flagging pairs above some threshold (e.g. |r| ≥ 0.5). Correlated pairs where one variable is essentially derived from or overlapping with another (e.g. three variables that are all different mathematical transforms of the same underlying quantity) are the clearest candidates for removing all but one.
- **Variance Inflation Factor (VIF)**: for each predictor, regress it on all the other predictors and compute $VIF = \dfrac{1}{1-R^2}$ (via `variance_inflation_factor` in `statsmodels`). A VIF of 1 means no overlap; 5 is a common warning threshold; 10 is often treated as a hard threshold. VIF captures multicollinearity among *more than two* variables at once (a variable that can be well predicted by a *combination* of several others, not just one), which pairwise correlation alone can miss.
- Multicollinearity is problematic because it inflates the standard errors of the coefficients involved, which can make a genuinely important predictor look statistically insignificant simply because a correlated "twin" is absorbing the credit; coefficient signs can also flip counterintuitively (e.g. one of two correlated predictors coming out with the "wrong" sign) purely because of the overlap, and resolve once the redundant predictor is dropped.

**Independence of observations**. Check that there are no duplicate rows (aside from an ID column) and no duplicate IDs — a copied row under a new ID would not be caught by an ordinary full-row duplicate check but would still let the same information appear in both train and test. If each row represents a separately measured, non-repeated subject/item with no time-series or repeated-measures structure, treating observations as independent is reasonable.

**Complete/quasi-complete separation**. If a categorical predictor's level perfectly (or almost perfectly) predicts the outcome — e.g. every single observation in one category has the same outcome value, with zero counter-examples — the logistic fit for that category's coefficient will not converge properly (it tries to push the coefficient to infinity, along with a nonsensical, enormous standard error), because that category alone perfectly explains the outcome with nothing to weigh against it. A crosstabulation (contingency table) of each categorical predictor against the target reveals this before fitting. The practical fix that costs the least information is to merge the perfectly-separating category with a chemically/logically similar category that behaves almost the same way (rather than discarding it, which would throw away real observations), so that there is at least one exception in the merged category for the model to weigh.

**Feature selection for the reduced model**. The same three techniques from Section 2 (univariate/chi-square, tree-based importance, correlation with target) can each be run on the full predictor set (after one-hot encoding categorical predictors), and their **ranks averaged** to combine them into a single ranking, since their raw scores are on incomparable scales. Where the three methods agree that a group of variables are all near the top and known (from the correlation/VIF check) to be redundant with each other, keep only one representative. Where a variable ranks poorly on the single-variable methods (chi-square, correlation) but highly on the combination-aware method (tree importance) and turns out to be a statistically significant predictor once fitted, that is direct evidence of a predictor whose effect only shows up in combination with others — exactly the blind spot of univariate methods, and a reason to keep it despite its poor single-variable ranking.

**Fitting for interpretation vs. fitting for prediction**. `sklearn.linear_model.LogisticRegression` is convenient for prediction/evaluation workflows but does not report p-values or standard errors. `statsmodels.api.Logit` (fit on the same, or an equivalently resampled, training data) is used specifically to get a coefficient table with standard errors, z/t-values, and p-values, needed to answer "which predictors are statistically significant."

**Threshold sensitivity**. Since logistic regression outputs a probability, the classification threshold (default 0.5) can be moved; lowering it increases recall (catches more positives, at the cost of more false positives / lower precision), raising it increases precision at the cost of recall. Recompute accuracy/precision/recall/F1 across several thresholds (e.g. 0.30, 0.40, 0.50, 0.60, 0.70) to see this trade-off directly, and pick the threshold appropriate to the cost of a false negative vs. a false positive in the application (e.g. if missing an unstable/at-risk case is costly, prefer a lower threshold that favors recall).

---

# 5. Similarity and Dissimilarity Measures

## 5.1 Definitions

- **Similarity measure**: a numerical measure of how alike two data objects are.
- **Dissimilarity measure**: a numerical measure of how different two data objects are.
- Dissimilarity ($d$) increases as objects become more different; identical objects give $d=0$.
- Similarity ($s$) increases as objects become more alike.
- For every **normalized** measure (bounded in $[0,1]$), $s = 1-d$; this does **not** hold for raw (unbounded) distances like plain Euclidean or Manhattan distance.
- Check the diagonal: a dissimilarity matrix has 0s down the diagonal; a similarity matrix has 1s.
- The matrix is symmetric ($d(i,j)=d(j,i)$), so only the lower (or upper) triangle needs to be filled; number of distinct pairs for $n$ objects: $\dfrac{n(n-1)}{2}$.

**Object** = one row (a thing being measured); **attribute** = one column (a property of that thing). $p$ in the formulas below is the number of attributes being compared for a given pair.

## 5.2 Picking the attribute type (decision order)

1. Does subtraction make sense on it (e.g. price, latency)? → **Numeric**.
2. Do the values have a fixed order but no meaningful distance (e.g. small/medium/large)? → **Ordinal**.
3. Are there only two possible values? → **Binary**.
4. Otherwise → **Nominal**.

## 5.3 Nominal attributes

Categories with no order; the only question is whether two values match.

$$d(i,j) = \frac{p-m}{p}$$

where $p$ = number of nominal attributes compared, $m$ = number that match. With one attribute, $d=0$ if same, $d=1$ if different. Similarity $s = 1-d = m/p$.

Worked pattern: for multiple nominal attributes, count matches column by column, divide mismatches by total attribute count.

## 5.4 Binary attributes

For a pair of objects, tally over all $b$ attributes:
- $f_{11}$: both are 1.
- $f_{10}$: first is 1, second is 0.
- $f_{01}$: first is 0, second is 1.
- $f_{00}$: both are 0.

These four always sum to $b$, the total attribute count.

| | Similarity | Dissimilarity |
|---|---|---|
| **Symmetric** | $\dfrac{f_{11}+f_{00}}{b}$ | $\dfrac{f_{10}+f_{01}}{b}$ |
| **Asymmetric (Jaccard)** | $\dfrac{f_{11}}{b-f_{00}}$ (Jaccard coefficient $J$) | $\dfrac{f_{10}+f_{01}}{b-f_{00}}$ |

- **Symmetric binary**: both outcomes (0 and 1) are equally meaningful/important (e.g. gender coded arbitrarily as 0/1 where neither value is inherently more informative).
- **Asymmetric binary**: one outcome (usually 1) is the "interesting"/rare one and the other (0) is the uninteresting default (e.g. presence of a rare disease). Two objects both being 0 on an asymmetric attribute is not treated as genuine evidence of similarity, so $f_{00}$ is dropped from both the numerator and denominator (asymmetric/Jaccard formulas). The same real-world attribute (e.g. gender) may be treated as symmetric or asymmetric depending on context/application.
- Subtracting a symmetric similarity from 1 gives the correct symmetric dissimilarity, and likewise for asymmetric — but you **cannot** convert a symmetric measure into an asymmetric one (or vice versa) just by subtracting from 1, since they have different denominators.
- If $f_{00}=0$ for a pair, the symmetric and asymmetric formulas give the same answer for that pair (nothing to subtract).

## 5.5 Ordinal attributes

Ranked categories: order is known but not the size of the gaps between ranks. Normalize the rank to $[0,1]$ first:

$$\hat a_i = \frac{i-1}{M-1} \quad (i = \text{rank position}, \; M = \text{number of distinct ranks})$$

The lowest rank always normalizes to 0, the highest to 1. E.g. 3 ranks → 0, 0.5, 1; 4 ranks → 0, 0.333, 0.667, 1.

**Single ordinal attribute**: $d = |\hat a_i - \hat a_j|$ (equivalently $s(x,y)=\sqrt{(\hat a_i - \hat a_j)^2}$ for a single attribute, which reduces to the absolute difference).

**Multiple ordinal attributes standing alone** (not inside a mixed-attribute problem): Euclidean distance over the normalized values, $d(i,j) = \sqrt{\sum_{f=1}^p (\hat a_{if} - \hat a_{jf})^2}$.

**Inside a mixed-attribute (Gower) problem**: each ordinal column instead contributes its own absolute difference, and the combining/averaging happens at the Gower step (Section 5.7) — do not use Euclidean there.

Ordinal columns are **always normalized before use, even when the ordinal column is the only attribute** — this is different from numeric columns (see next section), which are normalized only inside a mixed problem.

## 5.6 Numeric attributes

**Two different situations, two different formulas — mixing them up is the single most common source of lost marks:**

**(a) One numeric column inside a mixed-attribute problem** — range-normalize using the whole column's max and min (not the pair being compared):

$$d = \frac{|x_i - x_j|}{x_{max} - x_{min}}$$

The denominator is fixed once for the whole column/matrix; if it appears to change cell to cell, something is wrong. The pair holding the column's actual max and min will always come out at exactly $d=1$ for that column — a useful sanity check.

**(b) All numeric columns at once, for a standalone numeric dataset** — the **Minkowski distance** family:

$$d(x,y) = \left(\sum_{k=1}^p |x_k - y_k|^r\right)^{1/r}$$

- $r=1$: **Manhattan distance** (city-block / taxicab distance): $d = \sum_k |x_k-y_k|$.
- $r=2$: **Euclidean distance**: $d = \sqrt{\sum_k (x_k-y_k)^2}$.
- $r \to \infty$: **Supremum / Chebyshev distance**: $d = \max_k |x_k - y_k|$ (just the single largest per-attribute difference — cheapest to compute, no summing needed).
- As a special case, when attribute values are restricted to $\{0,1\}$, Manhattan distance becomes the **Hamming distance** (number of differing bits/positions).

**Which to use where**: range normalization (a) when the numeric column is one ingredient of a Gower average; Euclidean (b, $r=2$) when given raw coordinates and asked for a standalone distance matrix (this is also what typically feeds into AGNES clustering); Manhattan or supremum only when specifically named.

**Properties of Minkowski-family metrics**:
1. **Non-negativity**: $d(x,y)\ge 0$ for all $x,y$; $d(x,y)=0$ only if $x=y$ (identity condition).
2. **Symmetry**: $d(x,y)=d(y,x)$.
3. **Triangle inequality**: $d(x,z) \le d(x,y)+d(y,z)$ for all $x,y,z$.

## 5.7 Mixed attributes: Gower's distance

When a table has columns of different types, combine per-column dissimilarities using **Gower's distance**:

$$d(i,j) = \frac{\sum_{f=1}^{p} w_f\, \delta_f\, d_f}{\sum_{f=1}^{p} w_f\, \delta_f}$$

- $d_f$: the per-column dissimilarity for attribute $f$, computed using **that column's own rule** — numeric uses the range formula (5.6a), nominal/binary use $0$ if same / $1$ if different, ordinal uses the normalized absolute difference.
- $w_f$: the weight for attribute $f$. With no weights given, every $w_f=1$ (do **not** invent weights if none are stated).
- $\delta_f$ (the "switch"): equals **0** when either (i) a value is missing for that pair on attribute $f$, or (ii) attribute $f$ is **asymmetric binary** and both objects are 0 on it; otherwise $\delta_f = 1$. A skipped column drops out of **both** the numerator and the denominator for that pair, so that pair's dissimilarity divides by fewer columns than other pairs — a smaller effective count, not a zero contribution.
- With no weights and nothing skipped, Gower's formula collapses to a plain average of the per-column dissimilarities.

**The four-step Gower procedure**:
1. Normalize whatever needs it: ordinal columns get the rank formula; numeric columns get the range formula; nominal and binary columns are already 0/1 and need no normalization.
2. Build one small per-column dissimilarity matrix, using that column's own rule.
3. For each pair of objects, sum the per-column values across all columns that count for that pair.
4. Divide by the number (or weighted count) of columns that counted for that pair.

Common trap: **do not** average two "block" results (e.g. a symmetric-binary block score and a Jaccard block score) with equal weight after computing them separately — the blocks may have different numbers of contributing columns, and the correct combination automatically weights by how many attributes each block actually contributed, which is what the per-attribute Gower method does on its own.

**If the objects are on completely mixed nominal/ordinal/binary types and the task is to feed a distance matrix to AGNES clustering**: build the matrix via Gower first, exactly as above, then hand the resulting matrix to AGNES unchanged — AGNES only ever reads a distance/dissimilarity matrix, it does not care where the numbers came from.

## 5.8 Non-metric similarity: cosine similarity

For complex objects (e.g. containing many symbolic entities like keywords/phrases, as in text/information retrieval), a non-metric similarity function is often more useful than a strict distance.

**Cosine similarity** measures the cosine of the angle $\theta$ between two non-zero vectors $x$ and $y$:

$$SC(x,y) = \frac{x \cdot y}{\|x\|\,\|y\|}$$

where $x\cdot y$ is the dot product and $\|x\|,\|y\|$ are the vector magnitudes (lengths).

- Ranges from -1 to +1 in general; commonly reported as 0 to 1 for non-negative vectors (e.g. word/term counts), where 0 = no similarity and 1 = perfect similarity (same direction, differing only in magnitude).
- $\theta = 0°$: vectors overlap (same orientation) → cosine similarity = 1 → maximally similar.
- $\theta = 90°$: perpendicular (orthogonal) vectors → cosine similarity = 0 → no shared direction/terms.
- $\theta = 180°$: opposite vectors → cosine similarity = -1.
- **Cosine dissimilarity**: $DC(x,y) = 1 - SC(x,y)$.

**Worked example** (numeric vectors): $x=\{3,2,0,5\}$, $y=\{1,0,0,0\}$. $x\cdot y = 3$. $\|x\|=\sqrt{9+4+0+25}=6.16$. $\|y\|=1$. $SC(x,y)=3/6.16=0.49$; $DC(x,y)=0.51$.

**Worked example** (text/document similarity): build a shared vocabulary across two documents, count term frequencies to get a vector per document, then compute dot product, magnitudes, and cosine similarity as above. Interpretation bands sometimes used: 1.0 = identical, 0.7–0.99 = highly similar, 0.4–0.69 = moderately similar, 0.0–0.39 = weakly similar.

Cosine similarity is widely used in text analysis, document comparison, search queries, and recommendation systems, because it measures orientation (shared relative term usage) rather than magnitude (raw document length).

## 5.9 Choosing the appropriate measure

**For similarity**: sensitivity to data scale, the specific task (cosine similarity for information retrieval/text mining, Jaccard similarity for clustering/recommendation systems), and robustness to noise/outliers (e.g. the Sørensen–Dice coefficient is less sensitive to noise) all matter.

**For dissimilarity**: different measures suit different data types (Hamming distance for binary/string data, Euclidean for continuous numeric data); the scale of the data matters (if one feature's range dwarfs another's, Euclidean may be a poor choice — normalize/standardize, or use Manhattan instead); the number of dimensions matters (for high-dimensional data, a more robust measure such as Mahalanobis distance may be preferable).

## 5.10 Quick reference: the recurring questions

- **Do I normalize the numeric column?** Only if the table also has a non-numeric column (i.e., inside a mixed/Gower problem). An all-numeric table uses raw values (typically feeding straight into Euclidean/Manhattan/Minkowski).
- **Do I normalize the ordinal column?** Always, every time, even when it stands alone.
- **Do I normalize nominal or binary columns?** Never — they are already 0 or 1.
- **Does similarity vs. dissimilarity change which formula/method I use?** No — it only changes what goes on top of the fraction (agreements vs. disagreements). Symmetric vs. asymmetric changes what goes on the bottom (the denominator).
- **Can I get asymmetric from symmetric by doing 1 minus the value?** No. Subtracting from 1 only flips the direction (similarity ↔ dissimilarity) within the *same* symmetric/asymmetric family; the two families have different denominators and are not interchangeable that way.
- Manhattan, Euclidean, and Supremum distances only appear in an **all-numeric** problem, never inside a mixed-attribute problem (there, numeric columns use the range-normalized per-column formula instead, as part of Gower's method).

---

# 6. Hierarchical Clustering: AGNES and DIANA

## 6.1 Cluster analysis basics

- **Cluster**: a collection of data objects that are similar/related to each other within the group, and dissimilar/unrelated to objects in other groups.
- **Cluster analysis**: finding similarities in data and grouping similar objects into clusters. It is **unsupervised learning** — there are no predefined class labels (learning by observation, not by labeled examples).
- Typical applications: as a stand-alone exploratory tool, or as a preprocessing step for other algorithms.

**Applications of cluster analysis**: data reduction (summarization for downstream regression/PCA/classification/association analysis; compression e.g. vector quantization in image processing); hypothesis generation and testing; prediction based on group characteristics; finding K-nearest neighbors (localizing search to a small number of clusters); outlier detection (outliers are often "far away" from any cluster).

**Basic steps to develop a clustering task**: feature selection (minimal redundancy, relevant to the task); choosing a proximity measure; defining a clustering criterion (cost function or rule); choosing a clustering algorithm; validating results (clustering tendency test); interpreting results in the context of the application.

**Considerations for cluster analysis**: partitioning criteria (single-level vs. hierarchical — multi-level hierarchical is often desirable); separation of clusters (exclusive, e.g. one customer to one region, vs. non-exclusive, e.g. a document in multiple classes); similarity measure basis (distance-based e.g. Euclidean, vs. connectivity-based e.g. density/contiguity); clustering space (full space for low-dimensional data vs. subspace clustering for high-dimensional data).

**Partitioning methods** (contrast with hierarchical): partition $n$ objects into $k$ clusters minimizing $E = \sum_{i=1}^k \sum_{p\in C_i} (d(p,c_i))^2$ where $c_i$ is the centroid/medoid of cluster $C_i$. Requires $k$ to be chosen in advance. Heuristic algorithms: **k-means** (each cluster represented by its center) and **k-medoids / PAM** (each cluster represented by one of its actual member objects).

## 6.2 Hierarchical clustering: AGNES vs DIANA

Hierarchical clustering builds a full hierarchy (tree) of nested clusters and does **not** require the number of clusters to be specified up front. Results are visualized with a **dendrogram**. It uses a distance matrix as the clustering criterion; it does not need $k$ as input, but does need a **termination condition**.

**AGNES (AGglomerative NESting)** — bottom-up:
1. Every observation starts as its own single-element cluster (a leaf).
2. At each step, the two most similar clusters are merged into one bigger cluster (a node).
3. Repeat until all points belong to one single cluster.
4. Result: a tree, displayed as a dendrogram.

**DIANA (DIvisive ANAlysis)** — top-down, the mirror image of AGNES:
1. Starts with the root: all observations in one single cluster.
2. At each step, the current cluster is split into two clusters chosen to be the most heterogeneous (diverse) split possible.
3. Repeat until every observation is its own cluster.

Both AGNES and DIANA can be visualized on the same dendrogram, read in opposite directions: AGNES builds it left-to-right/bottom-up (merging), DIANA reads it as if unbuilding it top-down (splitting). With $n$ objects there are always exactly $n-1$ merge (or split) steps.

## 6.3 The dendrogram

The tree AGNES/DIANA produces. Objects sit along the bottom (leaves). Every merge draws a horizontal bar joining two branches; the **height of that bar is the distance at which the two branches merged**.

- A low bar means the two things it joins merged early — they are close/similar.
- A tall bar means the two things it joins were far apart and only merged because nothing closer was left.
- **Cutting the tree**: draw a horizontal line at some height; the number of vertical lines the cut crosses equals the number of clusters at that level; each vertical line carries one group of leaves below it.
- To get exactly $k$ clusters from $n$ objects, cut just below merge number $n-k$ (i.e., just below the $(n-k+1)$-th merge).
- **Choosing where to cut**: look for the single largest vertical gap between consecutive merge heights, and cut inside that gap. A large gap means the next merge had to reach much further than the previous ones to find anything to join, which is the data itself signaling that the groups on either side of that gap are genuinely separate/natural clusters.

## 6.4 Linkage methods (how distance between clusters is defined)

Once two objects merge into a cluster, "distance from that cluster to something else" needs a rule, because a cluster is no longer a single point.

- **Single link (nearest-neighbor / minimum linkage)**: distance between two clusters = the **smallest** distance between any member of one and any member of the other:
$$d(K_i,K_j) = \min\{d(p,q) : p\in K_i,\, q\in K_j\}$$
A cluster is treated as "near" another if *any one* member is near it. Produces **chaining** — long, straggly clusters formed one short link at a time; the dendrogram tends to show several low merges and one late, disproportionately tall one.

- **Complete link (furthest-neighbor / maximum linkage)**: distance between two clusters = the **largest** distance between any member of one and any member of the other:
$$d(K_i,K_j) = \max\{d(p,q) : p\in K_i,\, q\in K_j\}$$
A cluster is only "near" another if *all* of its members are near. Produces compact, tight clusters; merge heights climb faster than single link on the same data.

- **Average linkage**: distance between clusters = the average of all pairwise member-to-member distances, $dist(K_i,K_j) = avg(t_{ip}, t_{jq})$.
- **Centroid linkage**: distance between the two clusters' centroids, $dist(K_i,K_j)=dist(C_i,C_j)$.
- **Medoid linkage**: distance between the two clusters' medoids (a medoid is a chosen, centrally located *actual* member object of the cluster, unlike a centroid which is a computed average point), $dist(K_i,K_j)=dist(M_i,M_j)$.
- Ward linkage (used e.g. in `scipy`/`sklearn` implementations): minimizes the increase in within-cluster variance when two clusters are merged.

**Crucial rule that trips people up**: for **both** single and complete linkage, you always **merge on the smallest number currently in the matrix** — merging never uses the maximum, even for complete linkage. **Only the update step differs between the two**: single linkage updates a rebuilt cell using the **smaller (min)** of the two old distances; complete linkage updates using the **larger (max)** of the two old distances. "Complete link merges on the biggest number" is a common wrong belief — false; it merges on the smallest, same as single link, and only the *update rule* uses the maximum.

## 6.5 The AGNES procedure, step by step

1. Write the starting distance matrix as a lower triangle (or full symmetric matrix) with labels on both edges.
2. **Find the smallest number in the current matrix.** Those two clusters/rows merge; that number becomes the height of that join on the dendrogram.
3. **Rebuild the matrix**: replace the two merged rows/columns with a single new row/column for the merged cluster.
4. **Fill each new cell using the chosen linkage rule**, working from the **original** pairwise distances between individual objects (not from intermediate rebuilt-matrix numbers, though taking min/max of the two most recent matrix cells gives an equivalent answer as long as it's done consistently — "min of mins is a min").
5. Repeat steps 2–4 until one cluster remains ($n-1$ merges total for $n$ objects).

**Where the update numbers come from**: if clusters A and C just merged, and you need the new distance from {A,C} to B, compare the *original* $d(A,B)$ against the *original* $d(C,B)$ and take the min (single link) or max (complete link).

**Building the starting matrix from raw coordinates** (when given points instead of a distance matrix): compute pairwise Euclidean distance, $d = \sqrt{\sum_{k=1}^p (x_k-y_k)^2}$, for every pair, then proceed with AGNES as usual.

**If the data has mixed attribute types** (not plain coordinates): build the starting matrix with **Gower's distance** (Section 5.7) first, then hand that matrix to AGNES unchanged.

### Worked illustration (single link vs. complete link on the same matrix)

Given a 5-object matrix (A, B, C, D, E) with a chosen set of distances, running single link and complete link on the *identical* starting matrix illustrates the key facts:

| Step | Single link | Complete link |
|---|---|---|
| 1 | first pair merges at some height $h_1$ | **same pair, same height $h_1$** |
| 2 onward | merges tend to occur at low, closely-spaced heights ("chaining": e.g. 4, 5, 6, 7) | merge heights climb faster and further apart (e.g. 4, 6, 9, 14) |

Three general facts this illustrates, always true:
1. **The very first merge is identical for both linkages**, because at that point every cluster is still a single object, so min and max of a one-vs-one comparison are the same number. If two people's first merges differ, one of them made an error.
2. **The final clustering groupings are not guaranteed to match** between single and complete link (they may coincide by coincidence on some datasets, but this is not general).
3. **Complete link finishes at or above single link's final height on the same data**, because complete link always keeps taking the larger of every pair being compared during updates. If a complete-link run's heights come out *lower* than the corresponding single-link run, min/max were swapped somewhere.

### Reading a merge sequence into a dendrogram

The running list of "which pair merged, at what height" **is** the dendrogram data; drawing the tree from that list is mechanical once the list is complete.

### Self-check list before trusting an AGNES answer

- With $n$ objects there are always exactly $n-1$ merges (so $n-1$ recorded heights).
- The first merge height is identical for single and complete link on the same starting matrix.
- Merge heights must be **non-decreasing** as the algorithm proceeds; a later merge height smaller than an earlier one signals an arithmetic mistake.
- Complete-link heights are always $\ge$ the corresponding single-link heights on the same data.
- The very last (final) merge height equals the smallest remaining value (single link) or largest remaining value (complete link) in the final 2×2 matrix before everything collapses into one cluster.
- Every original object appears exactly once along the bottom of the dendrogram.

### Common mistakes (in rough order of frequency)

1. Merging on the maximum for complete link — wrong; merging is always on the minimum, only the *update* uses the maximum.
2. Reading the update value from the most recently rebuilt matrix instead of going back to the original pairwise distances when unsure.
3. Forgetting to rebuild the matrix after a merge and continuing with the stale one.
4. Leaving a row/column in the matrix for an object that has already been absorbed into a merged cluster.
5. Losing track of which objects belong to which cluster — write out the full cluster membership in the row label (e.g. "15,17"), not just a single letter.
6. Drawing dendrogram bars at the wrong heights — the bar height is the *merge distance*, not the step number.
7. Stopping before one cluster remains, unless a specific number of clusters was requested.
8. **Ties**: if two candidate pairs share the smallest value and involve no common object, either merge order gives the same final tree — pick one and note the choice. If they *do* share an object, pick one, state the choice explicitly, and continue.

## 6.6 DIANA (divisive) procedure

DIANA is the "inverse order" of AGNES — it starts with everything in one cluster and eventually each node forms its own cluster.

**Terminology**: dissimilarity between two points $|x_i - x_j|$ (for 1-D data; in general, any dissimilarity measure). **Diameter** of a cluster = the biggest pairwise dissimilarity between any two points currently in it.

**Algorithm** (one round):
1. Among all current clusters, find the one with the **largest diameter** — that is the cluster to split next.
2. Within that cluster, find the point with the **highest average dissimilarity** to the rest of the cluster's members; pull it out as a new one-point group, the **splinter group**.
3. **Reassignment loop**: for every other point remaining in the original cluster, compare its average distance to the (growing) splinter group against its average distance to the rest of the original cluster. If it is (on average) closer to the splinter group, move it into the splinter group.
4. Repeat step 3 (recomputing averages each time the splinter group gains a member) until no more points switch sides. The original cluster has now split into two.
5. Repeat the whole process (step 1) on the new set of clusters, splitting the cluster with the largest diameter each round, until every point is its own cluster (or until a stopping point/desired number of clusters is reached).

**Worked example** (dataset {2, 5, 9, 15, 16, 25}, dissimilarity = $|x_i-x_j|$):

*Step 1*: Whole cluster {2,5,9,15,16,25} has diameter $|25-2|=23$. Average distance of each point to the other five: 2→12, 5→9.6, 9→8, 15→8, 16→8.4, 25→15.6. Point 25 has the highest average (15.6), so it becomes the splinter: splinter = {25}, rest = {2,5,9,15,16}. Checking every remaining point's average distance to the rest of its group vs. distance to 25 shows none of them are closer to 25, so no one switches. Result after step 1: {2,5,9,15,16} and {25}.

*Step 2*: {25} has diameter 0 (single point). {2,5,9,15,16} has diameter $|16-2|=14$, so it is split next. Recomputed averages (excluding 25): 2→9.25, 5→7, 9→6, 15→7.5, 16→8.25. Point 2 has the highest average, so splinter = {2}, rest = {5,9,15,16}. Checking: point 5's average distance to the rest {9,15,16} (8.33) is worse than its distance to splinter {2} (3), so **5 moves** to the splinter. Splinter becomes {2,5}; recheck the remaining points {9,15,16} against the new splinter: point 9's average distance to rest {15,16} (6.5) is worse than to splinter {2,5} (5.5), so **9 moves** too. Splinter becomes {2,5,9}; recheck {15,16}: neither is closer to the splinter than to each other, so no further moves. Result: {2,5,9} and {15,16}.

*Final clusters* (if stopped at 3 clusters): {2,5,9}, {15,16}, {25}.

This example shows the reassignment loop is genuinely iterative — a point can be pulled into a splinter group only after it has grown, not necessarily on the first pass.

## 6.7 Distance between clusters — summary formulas

Given clusters $K_i, K_j$ with members $t_{ip}, t_{jq}$ respectively:
- Single link: $dist(K_i,K_j) = \min(t_{ip}, t_{jq})$
- Complete link: $dist(K_i,K_j) = \max(t_{ip}, t_{jq})$
- Average: $dist(K_i,K_j) = avg(t_{ip}, t_{jq})$
- Centroid: $dist(K_i,K_j) = dist(C_i, C_j)$
- Medoid: $dist(K_i,K_j) = dist(M_i, M_j)$ (medoid = a real, centrally-located member object, not a computed average point)

**Centroid, radius, and diameter of a single cluster** (for $N$ points $t_{ip}$ in the cluster):

- **Centroid** (the "middle" of the cluster): $C_m = \dfrac{\sum_{i=1}^N t_{ip}}{N}$
- **Radius** (square root of the average squared distance from any point to the centroid): $R_m = \sqrt{\dfrac{\sum_{i=1}^N (t_{ip}-c_m)^2}{N}}$
- **Diameter** (square root of the average squared pairwise distance between all points in the cluster): $D_m = \sqrt{\dfrac{\sum_{i=1}^N\sum_{j=1}^N (t_{ip}-t_{iq})^2}{N(N-1)}}$

## 6.8 Density-based clustering

Relies on a density-based notion of a cluster: a cluster is a **maximal set of density-connected points**. Discovers clusters of arbitrary shape and handles noise, unlike centroid/linkage-based methods.

**Density-reachable**: point $p$ is density-reachable from point $q$ (w.r.t. parameters `Eps`, `MinPts`) if there is a chain of points $p_1,\dots,p_n$ with $p_1=q$, $p_n=p$, such that each $p_{i+1}$ is directly density-reachable from $p_i$ — i.e., $p$ has a core point in its neighborhood, connected through a series of core points.

**Density-connected**: points $p$ and $q$ are density-connected (w.r.t. `Eps`, `MinPts`) if there exists a point $o$ such that both $p$ and $q$ are density-reachable from $o$.

**DBSCAN** (Density-Based Spatial Clustering of Applications with Noise): defines a cluster as a maximal set of density-connected points, using two parameters — `Eps` (neighborhood radius) and `MinPts` (minimum points required in a neighborhood for a point to count as a "core" point). Points are classified as **core** (has at least `MinPts` neighbors within `Eps`), **border** (within `Eps` of a core point but not itself core), or **outlier/noise** (neither).

**OPTICS** (Ordering Points To Identify the Clustering Structure): produces a special ordering of the database reflecting its density-based clustering structure; this ordering encodes information equivalent to the density-based clusterings for a broad range of parameter settings, so it is useful for both automatic and interactive cluster analysis, including discovering intrinsic clustering structure. Can be represented graphically/visually.

## 6.9 Grid-based clustering

Uses a multi-resolution grid data structure over the data space.

- **STING** (STatistical INformation Grid approach): the spatial area is divided into rectangular cells at several levels of resolution; each high-level cell partitions into smaller cells at the next level down. Statistical info per cell (count, mean, standard deviation, min, max, distribution type) is precomputed and stored, and higher-level cell parameters can be derived from lower-level ones. Queries are answered top-down: start from a pre-selected layer (typically few cells) and compute confidence intervals per cell.
- **CLIQUE**: both grid-based and subspace clustering.
- **WaveCluster**: a multi-resolution clustering approach using the wavelet transform.

## 6.10 Measuring clustering quality

Three kinds of measures: **external**, **internal**, **relative**.

- **External** (supervised — needs ground-truth labels): compares a clustering result against known/expert-specified class labels.
  - **Matching-based**: purity, F-measure.
  - **Pairwise**: based on counting pairs of samples assigned the same/different clusters under the true vs. predicted clusterings (TP, FN, FP, TN in a pairwise sense) — Jaccard coefficient, Rand statistic, Fowlkes-Mallows measure.
  - **Correlation-based**: discretized Huber statistic, normalized discretized Huber statistic.
  - **Rand index**: measures similarity of the true and predicted cluster assignments, ignoring permutations of cluster labels.
  - **Mutual Information based scores**: measure agreement between true and predicted assignments, ignoring label permutations.
  - **Homogeneity / completeness / V-measure** (conditional-entropy based): **homogeneity** = each predicted cluster contains only members of a single true class; **completeness** = all members of a given true class are assigned to the same predicted cluster.
  - **Fowlkes-Mallows score**: geometric mean of pairwise precision and recall.
- **Internal** (unsupervised — no ground truth needed): evaluates goodness by how well-separated and how compact the clusters are.
  - **Silhouette coefficient**: higher = better-defined clusters.
  - **Calinski-Harabasz index** (Variance Ratio Criterion): higher = better-defined clusters.
  - **Davies-Bouldin index**: lower = better separation between clusters.
- **Relative**: directly compares different clusterings, typically from different parameter settings of the same algorithm.

**Contingency matrix**: reports the intersection cardinality (count of shared members) for every true-cluster/predicted-cluster pair. **Pair confusion matrix**: a 2×2 matrix summarizing agreement/disagreement across all pairs of samples between two clusterings.

## 6.11 Applied methodology notes (from Prelim Activity 4)

General workflow for applying AGNES to a real (multi-feature) dataset:

- **Standardization is important** before computing distances whenever features are on very different natural scales (e.g. signal strength in dBm vs. throughput in Mbps vs. percentage fields) — otherwise whichever feature happens to have the largest raw numeric range will dominate the Euclidean distance calculation, exactly as with PCA. Z-score standardization (or, in the sample script, simple row-wise normalization) is applied to the numeric clustering features before building the distance matrix.
- Categorical columns that are not part of the numeric clustering features (e.g. geographic region, technology generation) are deliberately **left out of the distance computation** and instead cross-tabulated against the resulting cluster labels *after* clustering, to see whether the discovered clusters correspond to any meaningful real-world grouping (regional or technological pattern) without having forced that grouping into the distance metric itself.
- A **dendrogram with a horizontal cut line drawn at a chosen height** (e.g. `plt.axhline(y=6, ...)`) is the standard way to visually justify a chosen number of clusters; the cut height should correspond to a large vertical gap in the merge heights (same reasoning as Section 6.3).
- `AgglomerativeClustering(n_clusters=k, linkage=..., metric=...)` in scikit-learn requires `metric='euclidean'` when `linkage='ward'` (Ward's method is only defined for Euclidean distance); other linkages (`'complete'`, `'average'`, `'single'`) can pair with other metrics (e.g. `'cosine'`, `'manhattan'`).
- After fitting, cluster labels are attached back to the original (unscaled) dataset as a new column, and each cluster's characteristics are summarized by computing the **mean of each original numeric feature per cluster** (a cluster profile table) — this is what turns abstract cluster numbers into an interpretable label (e.g. "high throughput, low latency, low packet loss" vs. "low throughput, high latency, high packet loss, high concurrent users" suggesting congestion).
- Comparing **different linkage methods** (single, complete, Ward) on the same data and dendrogram is a standard way to check whether the "natural" number of clusters is robust to the choice of linkage, or whether it depends heavily on which linkage is used — single linkage in particular is prone to chaining (Section 6.4) and can produce a much less clean-looking dendrogram/split than complete or Ward linkage on the same data.
- **PCA can be combined with clustering purely for visualization**: reduce a high-dimensional feature space (e.g. seven clustering features) down to two principal components (Section 3) purely so the resulting clusters can be plotted and visually inspected in 2D, with points colored by their assigned cluster label — this is a visualization aid and is separate from (does not replace) the actual clustering, which is still done on the full standardized feature set.

---

# 7. Decision Flowcharts: choosing the right method

A compact decision procedure for picking the correct similarity/dissimilarity formula and clustering approach.

**1. What does the problem hand you?** Decide up front whether you have raw coordinates/points, an existing distance/similarity matrix, or a table of mixed attribute types — this determines which half of the material applies.

**2. One attribute type, or mixed?** Look at the *entire* table before computing anything. This decision determines whether you build **one** distance matrix directly, or build **several small per-column matrices and combine them with Gower** (Section 5.7). Do this check before doing any arithmetic.

**3. What type is each column?** Run the Section 5.2 decision order (numeric → ordinal → binary → nominal) on every column individually. The type routes you to that type's formula.

**4. Binary columns**: two independent choices need to be made — (a) symmetric or asymmetric (a property of the data/domain, decided by whether both outcomes are equally meaningful), and (b) similarity or dissimilarity (decided by what the question actually asks for). Write $b$ for the total attribute count; $b - f_{00}$ is the same thing as $f_{11}+f_{10}+f_{01}$, just computed a shorter way.

**5. Ordinal columns**: always get normalized via $\hat a_i = (i-1)/(M-1)$, even when it is the only column present — this is the one attribute type that is normalized unconditionally.

**6. Numeric columns**: normalizing happens **only** in the mixed-attribute case, **not** when every column is numeric (this is the point most often gotten backwards). All-numeric data uses raw values directly in Manhattan/Euclidean/Supremum (Minkowski family), which never appear inside a mixed-attribute (Gower) problem — there, numeric columns instead use the per-column range-normalized formula.

**7. Mixed attributes (Gower)**: run the Gower formula once per pair of objects; the "skip rule" (dropping a column when a value is missing, or when it's an asymmetric-binary column with both objects at 0) is the part most often missed.

**8. AGNES**: one repeated loop — find smallest current value, merge, rebuild matrix, refill using the linkage rule. Linkage choice only changes **how the rebuilt matrix is filled** (min for single link, max for complete link); it never changes the merge rule itself (always merge on the smallest current value).

**Frequently confused points, restated directly**:
- Normalize the numeric column only if the table has a non-numeric column in it; an all-numeric table uses raw values.
- Normalize the ordinal column always, even alone.
- Never normalize nominal or binary columns — they are already 0 or 1.
- Similarity vs. dissimilarity only changes the numerator (what's being counted: agreements or disagreements); symmetric vs. asymmetric only changes the denominator (how many attributes count at all).
- You cannot convert symmetric to asymmetric (or the reverse) by computing 1 minus the value — that only flips similarity/dissimilarity within the same family, since the two families have different denominators.
- Complete link does **not** merge on the biggest number. Both single and complete linkage merge on the smallest number in the current matrix; complete link only differs in using the maximum when *filling in* the rebuilt matrix afterward.
