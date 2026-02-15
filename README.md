# About

**Existing calculators don't (or incorrectly) take into account your bound mats, especially bound juice & books. This calculator tells you how to best use free taps and juice. It saves anywhere from 1% to 20%+ gold, depending on your use case and what you compare it to.**

Try it here! <https://honing-forecast.pages.dev/>  *(It works better on desktop)*  

## How to use

**1. Tick your upgrades, and input how much **untradable** mats you have.**

![Inputs](<https://i.redd.it/hyv473ghlphg1.png?width=1117&format=png&auto=webp&s=21d1ad94740acfbbd258f772281df2b692942e1c>)

**2. Press the big yellow button. Once it's done, it'll tell you how much things will cost.**

![Cost distribution](https://i.redd.it/mqu1pelvlphg1.png?width=1207&format=png&auto=webp&s=2252f4b4086c582dbc8f31e59ad72f0fb6821eb8)

**3. And here's how to use your free taps & juices:**

![Juice instructions](https://i.redd.it/nmgydnrylphg1.png?width=1009&format=png&auto=webp&s=f5856fe8367b2f3f9c14f8f4a0f96f7dcb9e9c5e)

**4. (Optional) As you fail/succeed taps, you can update your progress and the optimizer will consider your new situation.**

## How it works

Check out the [Details](/DETAILS.md)

## Contributing

You'll need the following:

1. [Rust](https://rust-lang.org/tools/install/) (this should be all you need if you just want to improve the [optimizer engine](/crates/arena/README.md))
2. [wasm-pack](https://drager.github.io/wasm-pack/installer/)
3. [Node.js](https://nodejs.org/en/download)

Run `npm install` for the dependencies, then `npm run dev` for the dev server.

You can take a look at [WIP](/WIP.md), but any help will be greatly appreciated!
