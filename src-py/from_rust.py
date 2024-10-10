import numpy as np
import time

def heavy_computation(size):
    """
    Simulates a heavy computation or initialization using NumPy.
    This can be generating a large random array or performing an intensive computation.
    """
    print("Starting heavy computation in Python...")
    
    # Simulate heavy computation by creating a large array and sleeping for a while
    data = np.random.rand(size, size)
    time.sleep(5)  # Simulating a heavy task taking 5 seconds
    
    print("Heavy computation in Python complete.")
    return data
