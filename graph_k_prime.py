import json
import numpy as np
import matplotlib.pyplot as plt

def scaled_sigmoid(min_value, max_value, var, budget, theta):
    range_ = max_value - min_value
    b = var / range_
    return range_ / (1.0 + np.exp(-b * theta)) + min_value

def main():
    with open('Kprime_graph_data.json', 'r') as f:
        data = json.load(f)

    # Convert to NumPy arrays
    data = np.array(data)
    x_values = data[:, 0]
    y_values = data[:, 1]

    y_min = y_values.min()
    y_max = y_values.max()

    # Vectorized evaluation
    approx = scaled_sigmoid(
        y_min,
        y_max,
        2154519.7785711614,
        0,
        x_values
    )

    # Create the plot
    plt.figure(figsize=(10, 6))
    plt.plot(x_values, y_values, 'black', linewidth=1)
    # plt.plot(x_values, approx, 'green', linewidth=1)

    plt.ylim(y_min, y_max)
    plt.xlabel('theta')
    plt.ylabel("K' - budget")
    plt.title("K'(t) - budget")
    plt.grid(True, alpha=0.3)
    plt.hlines(
        y=0,
        xmin=x_values.min(),
        xmax=x_values.max(),
        colors='blue',
        linestyles='-'
    )

    plt.tight_layout()
    plt.show()

if __name__ == '__main__':
    main()
