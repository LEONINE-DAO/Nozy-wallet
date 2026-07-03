use crate::error::TauriError;
use nozy::AddressBook;
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize)]
pub struct AddressBookEntryResponse {
    pub name: String,
    pub address: String,
    pub created_at: String,
    pub last_used: Option<String>,
    pub usage_count: u32,
    pub notes: Option<String>,
}

fn entry_to_response(entry: nozy::AddressEntry) -> AddressBookEntryResponse {
    AddressBookEntryResponse {
        name: entry.name,
        address: entry.address,
        created_at: entry.created_at.to_rfc3339(),
        last_used: entry.last_used.map(|t| t.to_rfc3339()),
        usage_count: entry.usage_count,
        notes: entry.notes,
    }
}

#[derive(Debug, Deserialize)]
pub struct AddAddressBookRequest {
    pub name: String,
    pub address: String,
    pub notes: Option<String>,
}

#[command]
pub async fn address_book_list() -> Result<Vec<AddressBookEntryResponse>, TauriError> {
    let book = AddressBook::new().map_err(|e| TauriError::from(e.to_string()))?;
    Ok(book
        .list_addresses()
        .into_iter()
        .map(entry_to_response)
        .collect())
}

#[command]
pub async fn address_book_add(request: AddAddressBookRequest) -> Result<(), TauriError> {
    let book = AddressBook::new().map_err(|e| TauriError::from(e.to_string()))?;
    book.add_address(request.name, request.address, request.notes)
        .map_err(|e| TauriError::from(e.to_string()))
}

#[command]
pub async fn address_book_remove(name: String) -> Result<bool, TauriError> {
    let book = AddressBook::new().map_err(|e| TauriError::from(e.to_string()))?;
    book.remove_address(&name)
        .map_err(|e| TauriError::from(e.to_string()))
}

#[command]
pub async fn address_book_get(name: String) -> Result<Option<AddressBookEntryResponse>, TauriError> {
    let book = AddressBook::new().map_err(|e| TauriError::from(e.to_string()))?;
    Ok(book.get_address(&name).map(entry_to_response))
}

#[command]
pub async fn address_book_search(query: String) -> Result<Vec<AddressBookEntryResponse>, TauriError> {
    let book = AddressBook::new().map_err(|e| TauriError::from(e.to_string()))?;
    Ok(book
        .search_addresses(&query)
        .into_iter()
        .map(entry_to_response)
        .collect())
}
