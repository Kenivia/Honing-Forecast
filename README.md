# About

Honing Forecast is a calculator that tells you how much a set of upgrades will cost and your likelihood of success.

**Try it on the [website](https://kenivia.github.io/Honing-Forecast/)!**

## Features

### Chance mode

- If I want to have x% chance of success, how much mats should I prep?

### Budget mode

- If I have this much mats, what are my odds of success?
- How should I use my juice & free taps? How much are they worth?
- What are my bottlenecks?

### Forecast mode

- What will my chances of success be in x weeks if I earn this much per week?
- What will be my bottlenecks in the future?
- How much gold will I need to spend on buying mats in x weeks?(Pessimistic estimate)

### Bonus: Gamba simulator

- A more hands-on feel of your prospects

## Work in Progress

- Better roster tracking & income estimation(I don't think automatic game-to-website input is possible)
- Input arithmetic parsing(e.g. allow inputs like 25*1234 for easier boxes calculation)
- Books & scroll calculations(Although for the most part just use them same way as juices)
- Ctrl z, delete
- Hover question mark tooltips
- MUCH more tests, need to actually test the components(like count_failure, tap_map and such), also need implement more integration tests
- Pre-compute prob_dist of groups of upgrades(such as +n all 5 armours etc)
- Matrix operation libraries to speed up monte carlo and what not? maybe go back to caching results?
- Confidence intervals(might have to actually learn some math first)
- Success chance optimization in juice/free tap usage(right now it just calculates a gold value, want to replace non-custom gold value with true highest success chance).

## Feedback

Found a bug, spotted an error or got a suggestion? Contact me @kenivia on Discord! Feedback is greatly appreciated.
