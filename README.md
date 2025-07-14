# DISCLAIMER
This is just a modified version of the original rsonpath project (version of the 14.06.2025) where I added code
to measure the skip behavior and time of the original algorithm.

Relevant changes were made to enable the "empty-list-opt" which helps for skipping very short brackets such as
{} and []. Tracking features were added to measure the time skipping alone on skipping sections of the algorithm.