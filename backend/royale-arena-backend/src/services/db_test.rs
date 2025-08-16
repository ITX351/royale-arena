#[cfg(test)]
mod tests {
    use dotenvy::from_filename;
    use crate::services::db::create_db_pool;

    #[test]
    fn test_database_connection() {
        // Load environment variables from .env.royale file
        let _ = from_filename(".env.royale");
        
        // Try to create a database connection pool
        let pool = create_db_pool();
        
        // Print the result for debugging
        println!("Database pool creation result: {:?}", pool);
        
        // We expect the pool to be created successfully
        assert!(pool.is_ok());
        
        // Try to get a connection from the pool
        let pool = pool.unwrap();
        let conn = pool.get_conn();
        
        // Print the result for debugging
        println!("Database connection result: {:?}", conn);
        
        // We expect the connection to be successful
        assert!(conn.is_ok());
        
        // Try to execute a simple query
        let mut conn = conn.unwrap();
        let result = conn.query::<u32, _>("SELECT 1");
        
        // Print the result for debugging
        println!("Database query result: {:?}", result);
        
        // We expect the query to be successful
        assert!(result.is_ok());
    }
}