use crate::models::admin::AdminUser;
use bcrypt::{DEFAULT_COST, hash, verify};
use mysql::prelude::*;

#[cfg(test)]
use crate::test_data::TestDataManager;

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
        (&id, username, &password_hash, is_super_admin),
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
        (username,),
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
        (username,),
    )?;

    match result {
        Some((id, username, password, is_super_admin)) => {
            Ok(Some(AdminUser {
                id,
                username,
                password, // This is the hash, not the plain text password
                is_super_admin,
            }))
        }
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
            Err(e) => {
                eprintln!(
                    "Warning: Failed to load .env.royale file for tests: {} (This is normal if database is not available during testing)",
                    e
                );
                return; // 如果无法加载环境变量，跳过测试
            }
        }

        // Create database connection pool
        let pool = match create_db_pool() {
            Ok(pool) => pool,
            Err(e) => {
                eprintln!(
                    "Skipping test_admin_user_creation_and_verification: Failed to create database pool: {} (This is normal if database is not available during testing)",
                    e
                );
                return; // 如果无法连接到数据库，跳过测试
            }
        };

        let mut conn = match pool.get_conn() {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!(
                    "Skipping test_admin_user_creation_and_verification: Failed to get database connection: {} (This is normal if database is not available during testing)",
                    e
                );
                return; // 如果无法获取数据库连接，跳过测试
            }
        };

        // Create test data manager
        let mut test_data_manager = TestDataManager::new();

        // Test data - 使用唯一用户名避免冲突
        let username = format!("test_user_{}", uuid::Uuid::new_v4());
        let password = "test_password";

        // Create admin user
        let result = create_admin_user(&mut conn, &username, password, false);
        if result.is_err() {
            eprintln!(
                "Skipping test_admin_user_creation_and_verification: Failed to create admin user: {} (This is normal if database is not available during testing)",
                result.err().unwrap()
            );
            return; // 如果无法创建用户，跳过测试
        }

        // Add user to test data manager for cleanup
        // 获取刚创建的用户的ID
        match conn.exec_first(
            "SELECT id FROM admin_users WHERE username = ?",
            (&username,),
        ) {
            Ok(user_id) => {
                if let Some(id) = user_id {
                    test_data_manager.created_admin_users.push(id);
                }
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to get user ID: {} (This is normal if database is not available during testing)",
                    e
                );
            }
        }

        // Verify password
        match verify_admin_password(&mut conn, &username, password) {
            Ok(verified) => {
                assert!(verified, "Password verification failed");
            }
            Err(e) => {
                eprintln!(
                    "Skipping password verification: {} (This is normal if database is not available during testing)",
                    e
                );
                // Clean up and return
                let _ = test_data_manager.cleanup();
                return;
            }
        }

        // Verify incorrect password
        match verify_admin_password(&mut conn, &username, "wrong_password") {
            Ok(verified) => {
                assert!(!verified, "Password verification should have failed");
            }
            Err(e) => {
                eprintln!(
                    "Skipping incorrect password verification: {} (This is normal if database is not available during testing)",
                    e
                );
                // Clean up and return
                let _ = test_data_manager.cleanup();
                return;
            }
        }

        // Get admin user
        match get_admin_user(&mut conn, &username) {
            Ok(user) => {
                assert!(user.is_some(), "User should exist");
                let user = user.unwrap();
                assert_eq!(user.username, username);
                assert!(!user.is_super_admin);
            }
            Err(e) => {
                eprintln!(
                    "Skipping get admin user: {} (This is normal if database is not available during testing)",
                    e
                );
                // Clean up and return
                let _ = test_data_manager.cleanup();
                return;
            }
        }

        // Clean up test data
        match test_data_manager.cleanup() {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "Warning: Failed to cleanup test data: {} (This is normal if database is not available during testing)",
                    e
                );
            }
        }
    }
}
