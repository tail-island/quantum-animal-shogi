try {
    cargo build --release

    try {
        Set-Location -Path crates\python
        maturin build --release
        Copy-Item ..\..\target\wheels\quantum_animal_shogi-0.1.0-cp313-cp313-win_amd64.whl ..\..\..\dist
    } catch {
        throw
    } finally {
        Set-Location ..\..
    }

    try {
        Set-Location -Path crates\wasm
        wasm-pack build --release
    } catch {
        throw
    } finally {
        Set-Location -Path ..\..
    }
} catch {
    Write-Error "Feiled..."
}
