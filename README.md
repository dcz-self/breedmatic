# Breedmatic

An experiment with evolving creatures.

The creatures breed and try to flood you. Whoever survives before being shot is allowed to breed again.

Based on [Kataster](https://github.com/Bobox214/Kataster).

## Running

Get assets from https://porcupinefactory.org/data/assets.zip and unpack them into the `assets` directory.

Then run the simulation:

```
cargo run --release
```

Let it run for a while. Interesting shooters evolve usually around the 100th attempt.

### Looking into the brain

The bottom left corner is the live view of the brain of the shooter.

The brain takes in signals from the two topmost circles: angle to nearest baddie, time alive. The signal passes along connections from top to bottom to neurons, and the final circles-neurons at the bottom result in the output signals: angle of the weapon, body turn rate, and movement speed.

Signal strength is expressed with color: gray means relaxed (0), yellow active (positive), blue negative. The stronger the signal is, the stronger the color.

Some connections have no circle at the top: those are "bias" connections. The source strength is always 1.

### Neurons

Every neuron's task is to sum up incoming signals, and then to activate based on the result. Most neurons activate proportionally, but there is also the sigmoid activation (result never exceeds [0, 1]), the step (anything below 0 turns into 0, anything above activates to 1), ReLU (anything below 0 is shunted to 0), and Gaussian (the farther the sum from 0, the farther the activation from 1).

So don't be surprised that a neuron with no connections is suddenly always active!

### Interpreting the text

Each round, some text will print on the console, informing about the current evolution. It looks like this:

```
Spawn offspring of 87
Spawned genotype Mut 20
Hidden
    Linear: -0.953 0.001 0.001 
    Linear: 0.000 -0.054 0.011 
    Gaussian: 0.000 0.070 0.687 
Out
    Linear: -0.853 0.000 -0.361 0.000 
    Linear: 0.000 -0.720 0.000 0.000 
    Step01: 0.000 1.190 0.000 0.000 

Wrote shooter.dot
Preserved as 90 with score 720
Pop 22
```

Let's break it down.

```
Spawn offspring of 87
```

Each shooter in the gene pool gets a unique number. This shows the number(s) of the parent(s) of the just-spawned shooter.

```
Spawned genotype Mut 20
```

The shooter went through 20 mutations. Mutations will happen more often when population is low.

```
Hidden
    Linear: -0.953 0.001 0.001 
    Linear: 0.000 -0.054 0.011 
    Gaussian: 0.000 0.070 0.687 
```

The hidden layer of neurons in the brain (rows). Inputs are: angle to baddie, time alive, bias (columns).

```
Out
    Linear: -0.853 0.000 -0.361 0.000 
    Linear: 0.000 -0.720 0.000 0.000 
    Step01: 0.000 1.190 0.000 0.000 
```

The output layer. Inputs come from the previous layer of neurons, plus an extra bias comes last (columns). Outputs are rows: weapon angle relative to movement direction, body turn speed, walk speed.

```
Wrote shooter.dot
```

The shooter was saved as a file for graphviz. Convert it to png using:

```
dot shooter.dot -Tpng -oshooter.png
```

Next comes the notice about shooter's death.

```
Preserved as 90 with score 720
Pop 22
```

The shooter's ID in the gene pool is 90 in this case, resulting in 22 different genotypes in the pool.

## License

The entire code may be distributed under AGPL 3.0 or higher, at your leisure. License text in the [`agpl-3.0.md` file](agpl-3.0.md).

In addition, remaining Kataster pieces are licensed under MIT. License text in the [`MIT.txt` file](MIT.txt).

https://www.rust-lang.org/

https://bevyengine.org/

https://www.rapier.rs/
