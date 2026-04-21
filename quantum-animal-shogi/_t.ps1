try {
    cargo build --release

    try {
        Set-Location -Path crates\python
        maturin develop --release
        python ..\..\_t.py
    } catch {
        throw
    } finally {
        Set-Location ..\..
    }
} catch {
    Write-Error "Feiled..."
}
