use crate::storage::Storage;
use std::collections::HashMap;
use std::sync::Arc;

pub trait Command: Send + Sync {
    fn execute(&self, args: Vec<String>, storage: Storage) -> String;
}

pub struct GetCommand;

impl Command for GetCommand {
    fn execute(&self, args: Vec<String>, storage: Storage) -> String {
        if args.len() != 1 {
            return "-ERR wrong number of arguments for 'get' command\r\n".to_string();
        }
        let key = &args[0];
        match storage.get(key) {
            Some(value) => format!("${}\r\n{}\r\n", value.len(), value),
            None => "$-1\r\n".to_string(),
        }
    }
}

pub struct SetCommand;

impl Command for SetCommand {
    fn execute(&self, args: Vec<String>, storage: Storage) -> String {
        if args.len() != 2 {
            return "-ERR wrong number of arguments for 'set' command\r\n".to_string();
        }
        let key = args[0].clone();
        let value = args[1].clone();
        storage.set(key, value);
        "+OK\r\n".to_string()
    }
}

pub struct CommandRegistry {
    commands: HashMap<String, Arc<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut commands = HashMap::new();
        commands.insert("GET".to_string(), Arc::new(GetCommand) as Arc<dyn Command>);
        commands.insert("SET".to_string(), Arc::new(SetCommand) as Arc<dyn Command>);
        CommandRegistry { commands }
    }

    pub fn get_command(&self, name: &str) -> Option<Arc<dyn Command>> {
        self.commands.get(&name.to_uppercase()).cloned()
    }
}

pub fn parse_request(buffer: &[u8]) -> (String, Vec<String>) {
    let request_str = String::from_utf8_lossy(buffer);
    let mut parts = request_str.trim().split_whitespace();
    let command = parts.next().unwrap_or("").to_uppercase();
    let args = parts.map(String::from).collect();
    (command, args)
}

pub fn execute_command(command_name: String, args: Vec<String>, storage: Storage, registry: Arc<CommandRegistry>) -> String {
    if let Some(command) = registry.get_command(&command_name) {
        command.execute(args, storage)
    } else {
        format!("-ERR unknown command '{}'\r\n", command_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    #[test]
    fn test_parse_request() {
        let (cmd, args) = parse_request(b"SET mykey myvalue\r\n");
        assert_eq!(cmd, "SET");
        assert_eq!(args, vec!["mykey", "myvalue"]);

        let (cmd, args) = parse_request(b"GET anotherkey\r\n");
        assert_eq!(cmd, "GET");
        assert_eq!(args, vec!["anotherkey"]);

        let (cmd, args) = parse_request(b"UNKNOWN_CMD arg1 arg2\r\n");
        assert_eq!(cmd, "UNKNOWN_CMD");
        assert_eq!(args, vec!["arg1", "arg2"]);

        let (cmd, args) = parse_request(b"  set   key   value  \r\n");
        assert_eq!(cmd, "SET");
        assert_eq!(args, vec!["key", "value"]);
    }

    #[test]
    fn test_get_command_execute() {
        let storage = Storage::new();
        storage.set("test_key".to_string(), "test_value".to_string());
        let get_cmd = GetCommand;

        let response = get_cmd.execute(vec!["test_key".to_string()], storage.clone());
        assert_eq!(response, "$10\r\ntest_value\r\n");

        let response = get_cmd.execute(vec!["non_existent_key".to_string()], storage.clone());
        assert_eq!(response, "$-1\r\n");

        let response = get_cmd.execute(vec![], storage.clone());
        assert_eq!(response, "-ERR wrong number of arguments for 'get' command\r\n");
    }

    #[test]
    fn test_set_command_execute() {
        let storage = Storage::new();
        let set_cmd = SetCommand;

        let response = set_cmd.execute(vec!["test_key".to_string(), "test_value".to_string()], storage.clone());
        assert_eq!(response, "+OK\r\n");
        assert_eq!(storage.get("test_key"), Some("test_value".to_string()));

        let response = set_cmd.execute(vec!["test_key".to_string()], storage.clone());
        assert_eq!(response, "-ERR wrong number of arguments for 'set' command\r\n");
    }

    #[test]
    fn test_command_registry() {
        let registry = CommandRegistry::new();
        assert!(registry.get_command("GET").is_some());
        assert!(registry.get_command("SET").is_some());
        assert!(registry.get_command("UNKNOWN").is_none());
        assert!(registry.get_command("get").is_some()); // Test case-insensitivity
    }

    #[test]
    fn test_execute_command() {
        let storage = Storage::new();
        let registry = Arc::new(CommandRegistry::new());

        // Test SET command
        let response = execute_command(
            "SET".to_string(),
            vec!["mykey".to_string(), "myvalue".to_string()],
            storage.clone(),
            registry.clone(),
        );
        assert_eq!(response, "+OK\r\n");
        assert_eq!(storage.get("mykey"), Some("myvalue".to_string()));

        // Test GET command
        let response = execute_command(
            "GET".to_string(),
            vec!["mykey".to_string()],
            storage.clone(),
            registry.clone(),
        );
        assert_eq!(response, "$7\r\nmyvalue\r\n");

        // Test unknown command
        let response = execute_command(
            "UNKNOWN".to_string(),
            vec![],
            storage.clone(),
            registry.clone(),
        );
        assert_eq!(response, "-ERR unknown command 'UNKNOWN'\r\n");
    }
}