
# Saddlepoint approximation

This document will go more into more detail about the math. We will first define what exactly we are computing, then go through how we do it.

## Setup

Unfortunately this thing is notationally a bit of a mess, but we will find that the math is rather simple up until the very last part. Before we get there we must first endure some boring definitions:

Let $U_j = U_1, U_2, \ldots , U_N$ be the upgrades we want to perform, which we will [view as discrete random variables](#simplification-of-an-upgrade-as-a-single-rv). Note that these take value in $0, 1, \ldots n_j$ and represent the number of attempts needed for each upgrade.

Let $X = (U_1, U_2, \ldots , U_N)$ be the random vector representing the overall outcome. Note that the upgrades are independent.

Let $C_i(x)$ be the **material** cost function for material $i$, where $i \in 1,2, \ldots, M$. For ordinary mats, this is of the form $C_i(x) = u_i + c_i \cdot x$ for some constants $u_i$ and $c_i$. However, this is arbitrary for juices.

Let $b_i$ be the "budget", or the amount of material $i$ that we own.

Let $m_i$ and $l_i$ be the **m**arket price and **l**eftover value of material $i$.

Let $g_i(x)$ be our **gold** cost function, defined as follows:

$$
g_i(x) =  \begin{cases}
  (x - b_i) \cdot   m_i  & \text{if } x > b_i \\
     (x - b_i) \cdot   l_i& \text{otherwise }
\end{cases}

$$

As in, every unit of material $i$ that we consume beyond our budget incurrs $m_i$ gold cost, and every unit of $i$ that we have leftover is worth $l_i$ gold.  By default, $l$ is 0 (our budget is untradable). [The signs are backwards](#opposite-signs).

As such, we can consider the **gold cost due to material** $i$:
$$
G_i(X) = g_i ( \sum^N_{j=1}C_i(U_j))
$$

Note that $g_i$ is applied to the sum, not the cost of the individual upgrades.

### Special honing

There's an additional layer of complexity we must define:

Let $U'_k$ be the "skipped" version of the random variable. This is just a deterministic constant equal to the number of normal honing attempts the user did before doing special hones, usually equal to 0 (but does not necessarily mean $C_i(U'_k) = 0$).

Let $X'_k = (U'_1, \cdots , U'_k, U_{k+1}, \cdots, U_N)$, the overall outcome with $k$ skipped upgrades.

Let $p_k$ be the chance that we skip exactly $k$ upgrades, where $k \in 0,1, \ldots, N$

### What we're computing

$$
\sum^N_{k=0} (p_k  \cdot \sum^M_{i=1}  \mathbb{E}[  G_i(X'_k)  ])
$$

This perhaps looks like a complicated way to say "Average gold cost". However, $G_i$ is non-linear, so we cannot factor this function out. And thus begins our journey of unwrapping this bundle of joy.

## Step 1: Special hones ($p_k$)

 We must recognize that we have made a simplification here - we assume that we must keep doing special hones on one upgrade until either we run out or we succeed. (The alternative being attemping A for a times, then B for b times, then back to A etc etc)

This assumption has a few motives / implications:

1. The decision space of special hones is just the permutations of $ U_1, U_2, \ldots , U_N$. This makes the optimizer's job a lot easier, even if this might forbid some better configurations.
2. Succeeding $k$ implies that we succeeeded $k-1$, which has 2 effects:
    1. This drastically reduces the number of possibilities we need to consider, and therefore the number of times we need to evaluate the inner sum. As in, we only need to consider $(fail, f )$, $(success, f)$, $(s, s)$, as opposed to the cartesian product $\{s,f\}^n$. This reduces the complexity from $2^n$ to $(n+1)$. This is by far the biggest concern.
    2. This makes the computation of $p_k$ a lot easier. This is done in [special.rs](/crates/core/src/special.rs) using some dynamic programming.
3. This is easier for the user to follow.

## Step 2: $ \mathbb{E}[  G_i(X'_k) ]$

Unfortunately, for a non-linear $f$,  $ \mathbb{E}[f(X)] \neq  f(\mathbb{E}[X])$, which is the whole reason why we're here. Recall that:
$$
G_i(X) = g_i ( \sum^N_{j=1}C_i(U_j))
$$

where $g_i$ is non-linear. If this was a general function, this would require either a (brute force) convolution of random variable or something like Fast Fourier Transform(I think?). However, we know that our $g_i$ is piecewise linear:

$$
g_i(x) =  \begin{cases}
  (x - b_i) \cdot   m_i  & \text{if } x > b_i \\
     (x - b_i) \cdot   l_i& \text{otherwise }
\end{cases}

$$

and so we can do something more clever. For convenience define
$$
Y_i=\sum^N_{j=1}C_i(U_j),\\
 \text{so } G_i(X) = g_i(Y_i).
$$
We can notice that :
$$
\mathbb{E}[g_i(Y_i)] = m_i \cdot \mathbb{E}[(Y_i-b_i) \cdot I( Y_i > b_i)] + l_i \cdot  \mathbb{E}[(Y_i-b_i) \cdot I( Y_i < b_i)],
$$
where $I$ is the indicator function. From here, we will drop the subscripts $_i$ as we're focusing on one material type dimension. We can split this into three parts ([average.rs](/crates/core/src/saddlepoint_approximation/average.rs)):
$$

\begin{align}
\mathbb{E}[g(Y)] &= m \cdot \mathbb{E}[Y-b] + (m-l) \cdot E[(Y-b) \cdot I(Y<B)] \nonumber\\
&= m \cdot \underbrace{\mathbb{E}[Y-b]}_{\text{part a}} + (m-l) \cdot (\underbrace{E[Y \cdot I(Y<B)]}_{\text{part b}} - \underbrace{E[b \cdot I(Y<B)]}_{\text{part c}})
\end{align}

$$

Where part $a)$ is thankfully trivial to calculate because $Y$ is a simple sum of RVs, making $\mathbb{E}$ distributive.

For part $b)$, we define the biased distribution $Y^*$, which has the probabily distribution $p^* = p \cdot y$ for each $y$ in the support of $Y$. In other words:

$$
\mathbb{P}(Y^* = y) = \frac{y \mathbb{P} (Y=y)}{\mathbb{E}[Y]},
$$
such that $Y^*$ satisfies:

$$
 \mathbb{E}[Y \cdot I( Y < b)] = \mathbb{P}(Y^* < b) \cdot \mathbb{E}[Y].
$$,

Part $c)$ is a simpler version of part $b)$, we have:

$$
 \mathbb{E}[b \cdot I( Y < b)] = \mathbb{P}(Y < b) \cdot b.
$$

**As such, all we need is some way to compute $\mathbb{P}(Y^* < b)$ and $\mathbb{P}(Y < b)$!**

## Step 3: $\mathbb{P}(Y < b)$

We use a procedure known as [**Saddleppoint approximation**](#saddlepoint-approximation-with-applications) to perform this computation. I will describe the steps below:

### Step 3.1: Cumulant generating function $K(s)$

We compute $K(s)$, where $K$ is different for a particular $Y$ but we will spare the subscripts. $K(s)$ is defined as follows:
$$
K(s) = \log(\mathbb{E}[ e^{sY}]).
$$
$K$ has the property that it is distributive over addition. Recall that $Y_i = \sum^N_{j=1}C(U_j)$, so we have:
$$
K(s) = \sum^N_{j=1}{\log(\mathbb{E}[ e^{s C(U_j)}])}.
$$
This property allows us to compute the individual summands and add them up at the end to obtain $K(s)$. Furthermore, it is also easy to compute its derivatives $K'(s)$, $K''(s)$, $K'''(s)$, $K^{(4)}(s)$ (we stop at the 4th cumulant for floating point reasons). This is done here: [cumulants.rs](/crates/core/src/saddlepoint_approximation/cumulants.rs).

### Step 3.2 Find $s$ such that $K'(s) \approx b$

Since we have access to higher derivatives of $K'(s)$, we can use a generalized version of newton's algorithm for root finding, known as [Householder's method](https://en.wikipedia.org/wiki/Householder%27s_method). This is done here: [root_finder.rs](/crates/core/src/root_finder.rs). My algorithm uses a bunch of heuristics regarding non-convergence that are complete trial and error, so there may be faster implementations.

### Step 3.3 $\mathbb{P}(Y<b) = $ magic formula

The derivation of the formula can be found in [Saddlepoint approximation with applications](#saddlepoint-approximation-with-applications), specifically the base form is stated in Section 1.2.1, the continuity corrected version in 2.4.4, and the limiting case in Section 5.2.3.
$$
\mathbb{P}(Y<b) = \begin{cases}\Phi(w) + \phi( w) (\frac{1}{w} − \frac{1}{ u})& \text{if } x \neq \mu  \\

\Phi(z) - \phi(z) \left[ \frac{\kappa_3}{6} H_2(z) + \frac{\kappa_4}{24} H_3(z) + \frac{\kappa_3^2}{72} H_5(z) \right] & \text{if } x \approx \mu \\

\end{cases}\\
\text{where }
w =  \text{sgn}(s)\sqrt{2 (sb^+-K(s))} \\
 u = 2  \delta^{-1}\sinh(s\delta/2)\sqrt{K''(s)} \\
 z = (b^+ - \mu)/\sigma\\
 \kappa_n = K^{(n)}(0)/\sigma^n \\
  \delta = \text{lattice span of the support of Y} \\
  b^+ = b + \delta/2 \\
  H_n = \text{Hermite polynomial of degree }n

$$

This is implemented in [saddlepoint_approximation.rs](/crates/core/src/saddlepoint_approximation/saddlepoint_approximation.rs).

### What about $Y^*$?

Naively computing the cumulant generating function of $Y^*$ would require a direct convolution, since $Y^*$ is not a sum of random variables and thus $K$ is not distributive over its summands. Fortunately, It turns out the cumulant of $Y^*$ and $Y$ can be related by the following:

$$
K_Y^*(s) = K_Y(s) + \log(K'_Y(s)) - \log(\mu)
$$

And higher cumulants can be found similarly via closed form formulae, as implemented in [cumulants.rs](/crates/core/src/saddlepoint_approximation/cumulants.rs). Note that all we know about this $Y^*$ is its cumulant generating function, but that is what SA was designed for.

## Where to find each part

The structure of the code that performs  saddlepoint approximation roughly follows the way I've described above. More specifically:

1. [average_gold_metric](/crates/core/src/saddlepoint_approximation/average.rs) calls [some preparation functions](/crates/core/src/normal_honing_utils.rs), computes / read from cache the [special honing probs](/crates/core/src/special.rs). This roughly corresponds to step 1 from above.
2. [one_dimension_average_gold](/crates/core/src/saddlepoint_approximation/average.rs) performs step 2,
3. [saddlepoint_approximation_wrapper](/crates/core/src/saddlepoint_approximation/saddlepoint_approximation.rs) evaluates the probabilities (step 3). It also routes to brute-force when the support of Y is below some threshold (50000).

## Addendum

### Simplification of an upgrade as a single RV

We simply the multi-step process into a single realization of the RV, so our goal is to find the "plan" that minimizes the average cost overall. Intuitively the 1st step of this plan is the optimal move, but there is some intricacies with the order we attempting things (talking about non-special hones here). I have elected to ignore such intricacies.

### Opposite signs

Note that in the document (and on the website), we consider gold cost due to mats needed as positive, and gold gained due to leftover mats as negative. In the code it's the other way round, for the sake of consistency with the UI we will consider costs as positive here.

### Saddlepoint approximation with applications

Butler, R.W., 2007. Saddlepoint approximations with applications (Vol. 22). Cambridge University Press.
