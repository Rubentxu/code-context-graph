"""
Example Python code for testing the Code Context Graph parser.
This file demonstrates various language constructs.
"""

import os
import sys
from typing import List, Dict, Optional
from dataclasses import dataclass


@dataclass
class User:
    """Represents a user in the system."""
    id: int
    name: str
    email: str
    active: bool = True

    def get_display_name(self) -> str:
        """Returns the display name for the user."""
        return self.name.title()

    def deactivate(self) -> None:
        """Deactivates the user account."""
        self.active = False
        print(f"User {self.name} has been deactivated")


class UserService:
    """Service for managing users."""
    
    def __init__(self):
        self.users: Dict[int, User] = {}
        self._next_id = 1

    def create_user(self, name: str, email: str) -> User:
        """Creates a new user."""
        user = User(id=self._next_id, name=name, email=email)
        self.users[user.id] = user
        self._next_id += 1
        return user

    def get_user(self, user_id: int) -> Optional[User]:
        """Retrieves a user by ID."""
        return self.users.get(user_id)

    def get_active_users(self) -> List[User]:
        """Returns all active users."""
        return [user for user in self.users.values() if user.active]

    def deactivate_user(self, user_id: int) -> bool:
        """Deactivates a user by ID."""
        user = self.get_user(user_id)
        if user:
            user.deactivate()
            return True
        return False


def main():
    """Main function demonstrating the user service."""
    service = UserService()
    
    # Create some users
    alice = service.create_user("Alice", "alice@example.com")
    bob = service.create_user("Bob", "bob@example.com")
    
    print(f"Created users: {alice.get_display_name()}, {bob.get_display_name()}")
    
    # Get active users
    active_users = service.get_active_users()
    print(f"Active users: {len(active_users)}")
    
    # Deactivate a user
    service.deactivate_user(alice.id)
    
    # Check active users again
    active_users = service.get_active_users()
    print(f"Active users after deactivation: {len(active_users)}")


if __name__ == "__main__":
    main()