# Proving Parameters Endpoints

Orchard spends require downloaded proving parameters.

Routes vary by api-server version — typical patterns:

- **GET** status — params present / missing
- **POST** download — trigger official parameter fetch

CLI:

```bash
nozy proving --status
nozy proving --download
```

Tauri: `check_proving_status`, `download_proving_parameters`.

See [Proving Parameters](../advanced/proving-parameters.md).
