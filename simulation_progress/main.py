import numpy as np

from matplotlib import pyplot as plt


FILE = "SIMULATION_PROGRESS_OUT.csv"


data = None
with open(FILE) as f:
    data = f.read()

lines = data.strip().split('\n')
time = []
points = []
point_caps = []
ppss = []
items = []

for line in lines:
    time_str, data = line.split(":")
    point_str, point_cap_str, pps_str, items_str = data.split(";")
    time.append(int(time_str))
    points.append(int(point_str))
    point_caps.append(int(point_cap_str))
    ppss.append(int(pps_str))
    items.append(eval(items_str))

items_np = np.array(items).T

# Plot
plt.figure(figsize=(12, 6))

sub_plot_rows = 2
sub_plot_columns = 2

# Plot value vs time
plt.subplot(sub_plot_rows, sub_plot_columns, 1)
plt.plot(time, points, label="Value", color="black")
plt.xlabel("Time")
plt.ylabel("Points")
plt.grid(True)

# Plot value vs time
plt.subplot(sub_plot_rows, sub_plot_columns, 2)
plt.plot(time, ppss, label="Value", color="black")
plt.xlabel("Time")
plt.ylabel("Points Per Second")
plt.grid(True)

# Plot each component of the vector
plt.subplot(sub_plot_rows, sub_plot_columns, 3)
for i, vec in enumerate(items_np):
    plt.plot(time, vec, label=f"vec[{i}]")
plt.xlabel("Time")
plt.ylabel("Vector Component Value")
plt.legend()
plt.grid(True)

# Plot value vs time
plt.subplot(sub_plot_rows, sub_plot_columns, 4)
plt.plot(time, point_caps, label="Value", color="black")
plt.xlabel("Time")
plt.ylabel("Point Cap")
plt.grid(True)

plt.tight_layout()
plt.show()
