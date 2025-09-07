// load-tests/spike-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '10s', target: 1000 },  // Sudden spike to 1000 users
    { duration: '1m', target: 1000 },   // Sustain
    { duration: '10s', target: 5000 },  // Spike to 5000 users
    { duration: '2m', target: 5000 },   // Sustain
    { duration: '30s', target: 0 },     // Ramp down
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500'], // Allow slower during spike
    'http_req_failed': ['rate<0.05'],   // Allow 5% errors during spike
  },
};

// ... rest same as scenarios.js