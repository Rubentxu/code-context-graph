package com.example;

import java.util.*;
import java.util.stream.Collectors;

/**
 * Example Java code for testing the Code Context Graph parser.
 * Demonstrates various language constructs and patterns.
 */
public class Example {
    
    public static class User {
        private final int id;
        private final String name;
        private final String email;
        private boolean active;
        
        public User(int id, String name, String email) {
            this.id = id;
            this.name = name;
            this.email = email;
            this.active = true;
        }
        
        public int getId() { return id; }
        public String getName() { return name; }
        public String getEmail() { return email; }
        public boolean isActive() { return active; }
        
        public String getDisplayName() {
            return Character.toUpperCase(name.charAt(0)) + name.substring(1);
        }
        
        public void deactivate() {
            this.active = false;
            System.out.println("User " + name + " has been deactivated");
        }
        
        @Override
        public String toString() {
            return String.format("User{id=%d, name='%s', email='%s', active=%s}", 
                               id, name, email, active);
        }
    }
    
    public static class UserService {
        private final Map<Integer, User> users = new HashMap<>();
        private int nextId = 1;
        
        public User createUser(String name, String email) {
            User user = new User(nextId++, name, email);
            users.put(user.getId(), user);
            return user;
        }
        
        public Optional<User> getUser(int userId) {
            return Optional.ofNullable(users.get(userId));
        }
        
        public List<User> getActiveUsers() {
            return users.values().stream()
                       .filter(User::isActive)
                       .collect(Collectors.toList());
        }
        
        public boolean deactivateUser(int userId) {
            Optional<User> user = getUser(userId);
            if (user.isPresent()) {
                user.get().deactivate();
                return true;
            }
            return false;
        }
        
        public void printUserStats() {
            long activeCount = users.values().stream()
                                   .filter(User::isActive)
                                   .count();
            System.out.printf("Total users: %d, Active: %d%n", 
                            users.size(), activeCount);
        }
    }
    
    public static void main(String[] args) {
        UserService service = new UserService();
        
        // Create some users
        User alice = service.createUser("Alice", "alice@example.com");
        User bob = service.createUser("Bob", "bob@example.com");
        User charlie = service.createUser("Charlie", "charlie@example.com");
        
        System.out.println("Created users:");
        System.out.println("- " + alice.getDisplayName());
        System.out.println("- " + bob.getDisplayName());
        System.out.println("- " + charlie.getDisplayName());
        
        // Print stats
        service.printUserStats();
        
        // Deactivate a user
        service.deactivateUser(alice.getId());
        
        // Print stats again
        service.printUserStats();
        
        // Show active users
        List<User> activeUsers = service.getActiveUsers();
        System.out.println("Active users:");
        activeUsers.forEach(user -> System.out.println("- " + user.toString()));
    }
}