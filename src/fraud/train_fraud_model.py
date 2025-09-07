# train_fraud_model.py

import pandas as pd
from sklearn.ensemble import IsolationForest
import joblib

# Load historical transactions (non-fraud labeled — unsupervised)
df = pd.read_csv("transactions.csv")

# Feature engineering
df['balance_ratio'] = df['amount'] / df['balance']
df['hour_of_day'] = pd.to_datetime(df['timestamp']).dt.hour
df['time_since_last_tx'] = df.groupby('user_id')['timestamp'].diff().dt.total_seconds().fillna(0)
df['tx_count_last_5min'] = df.groupby('user_id').rolling('5min', on='timestamp').size()

# Select features
features = ['amount', 'time_since_last_tx', 'tx_count_last_5min', 
            'balance_ratio', 'is_new_device', 'is_new_ip', 'hour_of_day']

X = df[features].fillna(0).values

# Train Isolation Forest
model = IsolationForest(contamination=0.01, random_state=42)  # expect 1% anomalies
model.fit(X)

# Save as ONNX
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType

initial_type = [('float_input', FloatTensorType([None, len(features)]))]
onx = convert_sklearn(model, initial_types=initial_type)
with open("fraud_model.onnx", "wb") as f:
    f.write(onx.SerializeToString())