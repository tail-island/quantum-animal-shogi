try {
    cargo build --release

    try {
        Set-Location -Path crates\python
        maturin develop --release
        python ..\..\_t.py
    } finally {
        Set-Location -Path ..\..
    }
} catch {
    Write-Error "Feiled..."
}
