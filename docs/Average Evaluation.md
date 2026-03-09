# Average Evaluation

The math behind this can be found in [the white paper](/docs/Saddlepoint%20Approximation.pdf). Here's the order that things occur in:

```mermaid
graph TD;
StateBundle --> Special[Update special probs];
State --> |Special state| StateBundle;
State[State chosen by optimizer] -->|State| Upgrade  --> udist[Update probability distribution] --> Adjust[Adjust for being skipped or not] -->|For each material type| Support["Support" object] --> SA[Saddlepoint Approximation] --> one_dimension_average_gold -->  Repeat;
Special --> |For each special outcome| Adjust;


Repeat[Add up gold averages, take weighted average over special outcomes];

```

Some points to note:

- The probability dist update is different from normal and advanced honing, adv honing has 3 different distributions for cost, juice and scroll
- We "collapse" the probability distribution by removing duplicates and 0 probability events.
- All of these distributions are considered "linear", as in the gap size between each non-zero prob support is constant.
