import numpy as np
import matplotlib.pyplot as plt
import json
from matplotlib.widgets import Slider
import matplotlib.cm as cm
import matplotlib.colors as colors
# Load data
data = np.array(json.load(open("./test_cases/brute_arrangement_test_4dc2dfb84301975b.json", "r")))

n, m,_ = data[0].shape
init_idx = 0

# Create meshgrid
x = np.arange(n)
y = np.arange(m)
X, Y = np.meshgrid(x, y, indexing='ij')

# Create figure
fig = plt.figure()
ax = fig.add_subplot(111, )

# Function to plot surface + minima
import numpy as np
import matplotlib.pyplot as plt

def plot_surface_with_minima(idx):
    """
    Plots Z with imshow and enables hover tooltip showing labels for the pixel under the cursor.
    - idx: index into `data` (same as your original)
    - ax: matplotlib Axes to draw on
    - fig: matplotlib Figure (used for connecting events)
    Returns the AxesImage (surf) and a function to disconnect the hover if desired.
    """

    # Prepare arrays (keep same structure as your original)
    Z = np.array([[float(y[0]) for y in z] for z in data[idx]])

    labels = np.array([[y[1] for y in z] for z in data[idx]], dtype=object)
    # Find minima (could be multiple)
    val_of_interest = np.max(Z) 
    if abs(val_of_interest) > 1:
        val_of_interest = np.min(Z)
    optima = np.argwhere(Z == val_of_interest)
    surf = ax.imshow(Z, cmap="viridis_r" if abs(val_of_interest) > 1 else "viridis" , origin="lower",
                 )

    # Single annotation object (tooltip) — start hidden
    annot = ax.annotate(
        text="", xy=(0,0), xytext=(10,10),
        textcoords="offset points",
        bbox=dict(boxstyle="round", fc="w"),
        arrowprops=dict(arrowstyle="->"),
        visible=False
    )

    # Track last displayed indices so we don't redraw unnecessarily
    last_idx = {'i': None, 'j': None}

    def on_move(event):
        # Only respond to motion over the correct axes
        if event.inaxes != ax:
            if annot.get_visible():
                annot.set_visible(False)
                fig.canvas.draw_idle()
            return

        # event.xdata/event.ydata can be None if outside data area
        if event.xdata is None or event.ydata is None:
            if annot.get_visible():
                annot.set_visible(False)
                fig.canvas.draw_idle()
            return

        # Map data coords to integer pixel indices.
        # With origin='lower', the first row of Z is at the bottom (y=0).
        j = int(np.floor(event.xdata + 0.5))  # column index
        i = int(np.floor(event.ydata + 0.5))  # row index

        # Bounds check
        nrows, ncols = Z.shape
        if i < 0 or i >= nrows or j < 0 or j >= ncols:
            if annot.get_visible():
                annot.set_visible(False)
                fig.canvas.draw_idle()
            return

        # If hovering same pixel as before, do nothing
        if last_idx['i'] == i and last_idx['j'] == j:
            return

        last_idx['i'], last_idx['j'] = i, j

        # Get label for this pixel
        label_text = str(labels[i, j])

        # Update annotation: position it at the data coordinates (j, i)
        annot.xy = (j, i)
        annot.set_text(label_text)
        annot.set_visible(True)

        # Optionally you can style text (font size, etc) here:
        # annot.get_bbox_patch().set_alpha(0.9)

        fig.canvas.draw_idle()

    # Also hide the annotation when leaving the axes
    def on_leave(event):
        if annot.get_visible():
            annot.set_visible(False)
            fig.canvas.draw_idle()
            last_idx['i'], last_idx['j'] = None, None

    cid_move = fig.canvas.mpl_connect("motion_notify_event", on_move)
    cid_leave = fig.canvas.mpl_connect("axes_leave_event", on_leave)

    # Return surf and a disconnect function so the caller can remove handlers if needed
    def disconnect():
        try:
            fig.canvas.mpl_disconnect(cid_move)
            fig.canvas.mpl_disconnect(cid_leave)
        except Exception:
            pass




    ax.scatter(
       optima[:, 1],
        optima[:, 0],
         
        color='red',
        s=40,
        label='Maxima'
    )

    ax.set_xlabel('a1')
    ax.set_ylabel('a2')
    if val_of_interest > 1:
        ax.set_title(f"Max = {np.max(Z):.0f} Min = {val_of_interest:.0f}" )
    else :
        ax.set_title(f"Max = {val_of_interest*100:.4f}% Min = { np.min(Z)*100:.4f}%")

    return surf, disconnect


# Initial plot
surf = plot_surface_with_minima(init_idx)

# Slider
ax_slider = plt.axes([0.25, 0.05, 0.5, 0.03])
slider = Slider(ax_slider, "prob", 0, len(data) - 1, valinit=init_idx, valstep=1)

# Update callback
def update(val):
    idx = int(slider.val)
    ax.clear()
    plot_surface_with_minima(idx)
    fig.canvas.draw_idle()

slider.on_changed(update)

plt.show()
