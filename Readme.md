# Rate limiter

## The Challenge - Building Your Own Rate Limiter

Token bucket - tokens are added to a ‘bucket’ at a fixed rate. The bucket has a fixed capacity. When a request is made it will only be accepted if there are enough tokens in the bucket. Tokens are removed from the bucket when a request is accepted.

Leaky bucket (as a queue) - The bucket behaves like a FIFO queue with a limited capacity, if there is space in the bucket the request is accepted.

Fixed window counter - record the number of requests from a sender occurring in the rate limit’s fixed time interval, if it exceeds the limit the request is rejected.

Sliding window counter - similar to the fixed window, but each request is timestamped and the window slides.