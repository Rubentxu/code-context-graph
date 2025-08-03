package com.example.inheritance;

import java.util.List;
import java.util.ArrayList;
import java.util.Optional;

/**
 * Complex inheritance example demonstrating multiple inheritance patterns
 * and modern Java features.
 */
public abstract class Animal {
    protected String name;
    protected int age;
    
    public Animal(String name, int age) {
        this.name = name;
        this.age = age;
    }
    
    public abstract void makeSound();
    
    public void eat() {
        System.out.println(name + " is eating");
    }
    
    public String getName() {
        return name;
    }
    
    public int getAge() {
        return age;
    }
}

interface Flyable {
    void fly();
    default void land() {
        System.out.println("Landing gracefully");
    }
}

interface Swimmable {
    void swim();
    
    static void diveDeep() {
        System.out.println("Diving to great depths");
    }
}

@FunctionalInterface
interface Predator {
    void hunt(Animal prey);
}

public class Bird extends Animal implements Flyable {
    private double wingspan;
    private String species;
    
    public Bird(String name, int age, double wingspan, String species) {
        super(name, age);
        this.wingspan = wingspan;
        this.species = species;
    }
    
    @Override
    public void makeSound() {
        System.out.println(name + " chirps melodiously");
    }
    
    @Override
    public void fly() {
        System.out.println(name + " soars with " + wingspan + "m wingspan");
    }
    
    public void migrate() {
        System.out.println(species + " is migrating south");
    }
    
    // Generic method
    public <T> void collectItems(List<T> items, T newItem) {
        items.add(newItem);
    }
}

public class Duck extends Bird implements Swimmable {
    private boolean canDive;
    
    public Duck(String name, int age, double wingspan, boolean canDive) {
        super(name, age, wingspan, "Duck");
        this.canDive = canDive;
    }
    
    @Override
    public void makeSound() {
        System.out.println(name + " quacks loudly");
    }
    
    @Override
    public void swim() {
        System.out.println(name + " paddles gracefully");
        if (canDive) {
            System.out.println(name + " dives underwater");
        }
    }
    
    // Method with lambda expression
    public void feedFlock(List<Duck> flock) {
        flock.forEach(duck -> {
            duck.eat();
            System.out.println(duck.getName() + " is well fed");
        });
    }
}

public class Eagle extends Bird implements Predator {
    private double taalonLength;
    
    public Eagle(String name, int age, double wingspan, double taalonLength) {
        super(name, age, wingspan, "Eagle");
        this.taalonLength = taalonLength;
    }
    
    @Override
    public void makeSound() {
        System.out.println(name + " screeches powerfully");
    }
    
    @Override
    public void hunt(Animal prey) {
        System.out.println(name + " hunts " + prey.getName() + " with " + taalonLength + "cm talons");
    }
    
    // Method with Optional and streams
    public Optional<Animal> findPrey(List<Animal> animals) {
        return animals.stream()
                .filter(animal -> animal.getAge() < 2)
                .filter(animal -> !(animal instanceof Eagle))
                .findFirst();
    }
}

// Utility class with static methods
public final class AnimalUtils {
    private AnimalUtils() {
        throw new UnsupportedOperationException("Utility class");
    }
    
    public static <T extends Animal> List<T> filterByAge(List<T> animals, int minAge) {
        return animals.stream()
                .filter(animal -> animal.getAge() >= minAge)
                .collect(Collectors.toList());
    }
    
    public static void demonstratePolymorphism() {
        List<Animal> animals = new ArrayList<>();
        animals.add(new Duck("Donald", 3, 0.8, true));
        animals.add(new Eagle("Baldy", 5, 2.2, 5.0));
        
        // Polymorphic behavior
        animals.forEach(Animal::makeSound);
        
        // Interface segregation
        animals.stream()
                .filter(animal -> animal instanceof Flyable)
                .map(animal -> (Flyable) animal)
                .forEach(Flyable::fly);
    }
}