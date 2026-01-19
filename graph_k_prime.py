import json
import matplotlib.pyplot as plt



def main():
    # Read the JSON file
    with open('Kprime_graph_data.json', 'r') as f:
        data = json.load(f)
    
    # Separate x and y coordinates
    x_values = [point[0] for point in data]
    y_values = [point[1] for point in data]
    f  = lambda x :  6620.120426958165 +  2154519.7785711614 *x + 0.5 *  607054414.6722229 * x*x - 12470

    approx = [f(point[0]) for point in data]
    
    # Create the plot
    plt.figure(figsize=(10, 6))
    plt.plot(x_values, y_values, 'black', linewidth=1, markersize=1)
    plt.plot(x_values, approx, 'green', linewidth=1, markersize=1)
    plt.ylim(min(y_values), max(y_values))
    plt.xlabel('theta')
    plt.ylabel("K' - budget")
    plt.title("K'(t) - budget")
    plt.grid(True, alpha=0.3)
    plt.hlines(y=[0],xmax=max(x_values), xmin=min(x_values), colors=['blue',], linestyles=['-'])
    # Show the plot
    plt.tight_layout()
    plt.show()

if __name__ == '__main__':
    main()
