use bcrypt::{hash, verify, DEFAULT_COST};
use mysql::prelude::*;
use crate::models::admin::AdminUser;

pub fn create_admin_user(
    conn: &mut mysql::PooledConn,
    username: &str,
    password: &str,
    is_super_admin: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let id = uuid::Uuid::new_v4().to_string();
    let password_hash = hash(password, DEFAULT_COST)?;
    
    conn.exec_drop(
        "INSERT INTO admin_users (id, username, password, is_super_admin) VALUES (?, ?, ?, ?)",
        (&id, username, &password_hash, is_super_admin)
    )?;
    
    Ok(())
}

pub fn verify_admin_password(
    conn: &mut mysql::PooledConn,
    username: &str,
    password: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let result: Option<(String,)> = conn.exec_first(
        "SELECT password FROM admin_users WHERE username = ?",
        (username,)
    )?;
    
    match result {
        Some((hash,)) => Ok(verify(password, &hash)?),
        None => Ok(false),
    }
}

pub fn get_admin_user(
    conn: &mut mysql::PooledConn,
    username: &str,
) -> Result<Option<AdminUser>, Box<dyn std::error::Error>> {
    let result: Option<(String, String, String, bool)> = conn.exec_first(
        "SELECT id, username, password, is_super_admin FROM admin_users WHERE username = ?",
        (username,)
    )?;
    
    match result {
        Some((id, username, password, is_super_admin)) => {
            Ok(Some(AdminUser {
                id,
                username,
                password, // This is the hash, not the plain text password
                is_super_admin,
            }))
        },
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::db::create_db_pool;
    use dotenvy::from_filename;
    
    #[test]
    fn test_admin_user_creation_and_verification() {
        // Load environment variables from .env.royale file
        match from_filename(".env.royale") {
            Ok(_) => println!("Successfully loaded .env.royale file for tests"),
            Err(e) => eprintln!("Warning: Failed to load .env.royale file for tests: {}", e),
        }
        
        // Create database connection pool
        let pool = create_db_pool().expect("Failed to create database pool");
        let mut conn = pool.get_conn().expect("Failed to get database connection");
        
        // Test data
        let username = "test_user";
        let password = "test_password";
        
        // Clean up any existing test user
        conn.exec_drop("DELETE FROM admin_users WHERE username = ?", (username,))
            .expect("Failed to clean up test user");
        
        // Create admin user
        let result = create_admin_user(&mut conn, username, password, false);
        assert!(result.is_ok(), "Failed to create admin user: {:?}", result.err());
        
        // Verify password
        let verified = verify_admin_password(&mut conn, username, password)
            .expect("Failed to verify password");
        assert!(verified, "Password verification failed");
        
        // Verify incorrect password
        let verified = verify_admin_password(&mut conn, username, "wrong_password")
            .expect("Failed to verify password");
        assert!(!verified, "Password verification should have failed");
        
        // Get admin user
        let user = get_admin_user(&mut conn, username)
            .expect("Failed to get admin user");
        assert!(user.is_some(), "User should exist");
        let user = user.unwrap();
        assert_eq!(user.username, username);
        assert!(!user.is_super_admin);
        
        // Clean up test user
        conn.exec_drop("DELETE FROM admin_users WHERE username = ?", (username,))
            .expect("Failed to clean up test user");
    }
}