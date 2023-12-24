# rust-sol-data-collector

### Optimum Number of Threads Calculation
Given the following inputs:
- $R_l$ (Rate Limit): The maximum number of requests allowed per time window.
- $W$ (Window of Time): The duration of the time window in milliseconds.
- $T$ (Average Time per Request): The average time it takes to pull blocks from the network in milliseconds.

The Optimum Number of Threads ($O_n$) can be calculated using the formula:

$$
O_n = \frac{R_l}{\min\left(\frac{W}{T}, R_l\right)}
$$

Where:
- $\frac{R_l}{\frac{W}{T}}$ calculates the effective number of requests that a single thread can handle within the time window $W$.
- $\min\left(\frac{W}{T}, R_l\right)$ ensures that the effective number of requests per thread does not exceed the rate limit.
- $O_n$ is rounded up to the nearest whole number as the number of threads must be an integer.
