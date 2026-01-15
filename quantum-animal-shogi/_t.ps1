cargo build --release && Set-Location -Path crates\python && maturin develop --release && Set-Location -Path ..\.. && python ..\_t.py
