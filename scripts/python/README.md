# Setup venv
```bash
python3 -m venv venv
source venv/bin/activate
pip3 install -r requirements.txt
```

# Running
`cli.py` expects newline-delimited json over stdin. For example:
```bash
 ssh <host> "journalctl --user -t kinet --since -1m --output cat --no-tail" | python3 cli.py
```

