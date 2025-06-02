use std::fs;
use std::io::{Write, stdin, stdout};
use std::str::FromStr;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! input {
    ($output_type:ty, $($format_args:tt)*) => {
        $crate::input::<$output_type>(std::format!($($format_args)*).as_str())
    };
    ($($format_args:tt)*) => {
        $crate::input(std::format!($($format_args)*).as_str())
    };
}

fn input<T: FromStr>(prompt_message: &str) -> Result<T, T::Err> {
    print!("{}", prompt_message);
    if let Err(e) = stdout().flush() {
        eprintln!("Warning: Could not flush output: {}", e);
    }
    let mut output = String::new();
    if let Err(e) = stdin().read_line(&mut output) {
        eprintln!("Warning: Could not read input: {}", e);
    }
    output.trim().parse()
}

#[derive(Deserialize, Serialize, Default, Clone)]
struct HabitTracker {
    daily_habit_records: Vec<DailyRecord>,
    tracked_habits: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
struct DailyRecord {
    record_date: String,
    habit_completions: Vec<HabitCompletion>,
}

#[derive(Deserialize, Serialize, Clone)]
struct HabitCompletion {
    habit_name: String,
    is_completed: bool,
}

impl HabitTracker {
    fn get_most_recent_date(&self) -> Option<&str> {
        self.daily_habit_records
            .last()
            .map(|record| record.record_date.as_str())
    }

    fn get_most_recent_record_mut(&mut self) -> Option<&mut DailyRecord> {
        self.daily_habit_records.last_mut()
    }

    fn create_new_daily_record(&mut self, date_string: String) {
        let habit_completions = self
            .tracked_habits
            .iter()
            .map(|habit_name| HabitCompletion {
                habit_name: habit_name.clone(),
                is_completed: false,
            })
            .collect();

        let new_record = DailyRecord {
            record_date: date_string,
            habit_completions,
        };

        self.daily_habit_records.push(new_record);
    }

    fn add_new_habit(&mut self, new_habit_name: String) -> Result<(), String> {
        let trimmed_habit = new_habit_name.trim();

        if trimmed_habit.is_empty() {
            return Err("Habit name cannot be empty or just whitespace".to_string());
        }

        if trimmed_habit.len() > 100 {
            return Err("Habit name is too long (maximum 100 characters)".to_string());
        }

        if self
            .tracked_habits
            .iter()
            .any(|existing_habit| existing_habit.to_lowercase() == trimmed_habit.to_lowercase())
        {
            return Err(format!(
                "A habit similar to '{}' already exists",
                trimmed_habit
            ));
        }

        self.tracked_habits.push(trimmed_habit.to_string());

        for daily_record in &mut self.daily_habit_records {
            daily_record.habit_completions.push(HabitCompletion {
                habit_name: trimmed_habit.to_string(),
                is_completed: false,
            });
        }

        Ok(())
    }

    fn remove_habit(&mut self, habit_to_remove: &str) -> bool {
        if let Some(habit_index) = self
            .tracked_habits
            .iter()
            .position(|h| h.to_lowercase() == habit_to_remove.to_lowercase())
        {
            let removed_habit = self.tracked_habits.remove(habit_index);

            for daily_record in &mut self.daily_habit_records {
                daily_record.habit_completions.retain(|completion| {
                    completion.habit_name.to_lowercase() != removed_habit.to_lowercase()
                });
            }

            true
        } else {
            false
        }
    }

    fn calculate_habit_statistics(&self, target_habit: &str) -> (usize, usize, f64) {
        let total_recorded_days = self.daily_habit_records.len();
        let completed_days = self
            .daily_habit_records
            .iter()
            .filter_map(|daily_record| {
                daily_record
                    .habit_completions
                    .iter()
                    .find(|completion| {
                        completion.habit_name.to_lowercase() == target_habit.to_lowercase()
                    })
                    .map(|completion| completion.is_completed)
            })
            .filter(|&is_completed| is_completed)
            .count();

        let success_percentage = if total_recorded_days > 0 {
            (completed_days as f64 / total_recorded_days as f64) * 100.0
        } else {
            0.0
        };

        (completed_days, total_recorded_days, success_percentage)
    }
}

fn save_tracker_data(habit_tracker: &HabitTracker) -> Result<(), String> {
    let serialized_data =
        ron::to_string(&habit_tracker).map_err(|e| format!("Failed to serialize data: {}", e))?;

    fs::write("habit_tracker.ron", serialized_data)
        .map_err(|e| format!("Failed to save to file: {}", e))?;

    Ok(())
}

fn load_tracker_data() -> HabitTracker {
    match fs::read_to_string("habit_tracker.ron") {
        Ok(file_contents) if !file_contents.trim().is_empty() => ron::from_str(&file_contents)
            .unwrap_or_else(|parse_error| {
                eprintln!(
                    "⚠️  Could not read saved habit data ({}), starting fresh",
                    parse_error
                );
                HabitTracker::default()
            }),
        _ => {
            println!("📝 No previous habit data found, starting with a clean slate!");
            HabitTracker::default()
        }
    }
}

fn ask_yes_no_question(question: &str) -> bool {
    loop {
        match input!(char, "{}? (y/n): ", question) {
            Ok('y' | 'Y') => return true,
            Ok('n' | 'N') => return false,
            Ok(_) => println!("Please enter 'y' for yes or 'n' for no."),
            Err(_) => println!("Invalid input. Please try again."),
        }
    }
}

#[derive(Debug, PartialEq)]
enum UserChoice {
    AddHabit = 1,
    UpdateTodaysHabits,
    ViewProgress,
    DeleteHabit,
    ViewDetailedStats,
    ExitProgram,
}

impl FromStr for UserChoice {
    type Err = ();
    fn from_str(choice_number: &str) -> Result<Self, Self::Err> {
        match choice_number {
            "1" => Ok(UserChoice::AddHabit),
            "2" => Ok(UserChoice::UpdateTodaysHabits),
            "3" => Ok(UserChoice::ViewProgress),
            "4" => Ok(UserChoice::DeleteHabit),
            "5" => Ok(UserChoice::ViewDetailedStats),
            "6" => Ok(UserChoice::ExitProgram),
            _ => Err(()),
        }
    }
}

fn display_main_menu() {
    println!(
        "🎯 === Personal Habit Tracker ===
    1. 📝 Add a new habit to track
    2. ✅ Update today's habit completions
    3. 📊 View habit progress summary
    4. 🗑️  Delete a habit
    5. 📈 View detailed statistics
    6. 👋 Exit program"
    );
}

fn handle_add_habit(habit_tracker: &mut HabitTracker) {
    println!("\n📝 Adding a new habit to track...");

    let new_habit_input: String = match input!("What new habit would you like to start tracking? ")
    {
        Ok(input) => input,
        Err(_) => {
            println!("❌ Invalid input. Please try again.");
            return;
        }
    };

    match habit_tracker.add_new_habit(new_habit_input.clone()) {
        Ok(()) => {
            println!("✅ Successfully added habit: '{}'", new_habit_input.trim());

            if ask_yes_no_question("Would you like to mark this habit as completed for today") {
                if let Some(today_record) = habit_tracker.get_most_recent_record_mut() {
                    if let Some(habit_completion) =
                        today_record
                            .habit_completions
                            .iter_mut()
                            .find(|completion| {
                                completion.habit_name.to_lowercase()
                                    == new_habit_input.trim().to_lowercase()
                            })
                    {
                        habit_completion.is_completed = true;
                        println!(
                            "🎉 Great! Marked '{}' as completed for today!",
                            new_habit_input.trim()
                        );
                    }
                }
            }
        }
        Err(error_message) => {
            println!("❌ Could not add habit: {}", error_message);
        }
    }
}

fn handle_update_habits(habit_tracker: &mut HabitTracker) {
    println!("\n✅ Updating today's habit completions...");

    if let Some(todays_record) = habit_tracker.get_most_recent_record_mut() {
        let mut habits_updated = false;
        let mut completed_count = 0;
        let total_habits = todays_record.habit_completions.len();

        for habit_completion in &mut todays_record.habit_completions {
            if habit_completion.is_completed {
                completed_count += 1;
                continue;
            }

            if ask_yes_no_question(&format!(
                "Did you complete '{}'",
                habit_completion.habit_name
            )) {
                habit_completion.is_completed = true;
                completed_count += 1;
                habits_updated = true;
                println!(
                    "🎉 Awesome! Marked '{}' as completed!",
                    habit_completion.habit_name
                );
            }
        }

        if habits_updated {
            println!(
                "📊 Today's progress: {}/{} habits completed!",
                completed_count, total_habits
            );
        } else if completed_count == total_habits && total_habits > 0 {
            println!("🌟 Amazing! All your habits for today are already completed!");
        } else {
            println!("📝 No changes made to today's habits.");
        }
    } else {
        println!("❌ No habit records found. This shouldn't happen!");
    }
}

fn handle_view_progress(habit_tracker: &HabitTracker) {
    println!("\n📊 === Habit Progress Summary ===");

    if habit_tracker.tracked_habits.is_empty() {
        println!(
            "📝 You haven't added any habits yet. Add some habits first to see your progress!"
        );
        return;
    }

    for habit_name in &habit_tracker.tracked_habits {
        let (completed_days, total_days, success_rate) =
            habit_tracker.calculate_habit_statistics(habit_name);

        let progress_bar = create_progress_bar(success_rate);
        let status_emoji = if success_rate >= 80.0 {
            "🔥"
        } else if success_rate >= 60.0 {
            "👍"
        } else if success_rate >= 40.0 {
            "📈"
        } else {
            "💪"
        };

        println!(
            "{} {:<30} {} {}/{} days ({:.1}%)",
            status_emoji, habit_name, progress_bar, completed_days, total_days, success_rate
        );
    }
}

fn create_progress_bar(percentage: f64) -> String {
    let bar_length = 20;
    let filled_length = ((percentage / 100.0) * bar_length as f64) as usize;
    let empty_length = bar_length - filled_length;

    format!(
        "[{}{}]",
        "█".repeat(filled_length),
        "░".repeat(empty_length)
    )
}

fn handle_delete_habit(habit_tracker: &mut HabitTracker) {
    println!("\n🗑️  Deleting a habit...");

    if habit_tracker.tracked_habits.is_empty() {
        println!("📝 No habits found to delete. Add some habits first!");
        return;
    }

    println!("Current habits you're tracking:");
    for (index, habit_name) in habit_tracker.tracked_habits.iter().enumerate() {
        println!("  {}. {}", index + 1, habit_name);
    }

    let habit_to_delete: String =
        match input!("\nWhich habit would you like to stop tracking? (enter the exact name): ") {
            Ok(input) => input,
            Err(_) => {
                println!("❌ Invalid input. Please try again.");
                return;
            }
        };

    if ask_yes_no_question(&format!(
        "Are you sure you want to delete '{}' and all its history",
        habit_to_delete.trim()
    )) {
        if habit_tracker.remove_habit(&habit_to_delete) {
            println!(
                "✅ Successfully deleted habit: '{}'",
                habit_to_delete.trim()
            );
        } else {
            println!(
                "❌ Habit '{}' not found. Make sure you entered the exact name.",
                habit_to_delete.trim()
            );
        }
    } else {
        println!("📝 Habit deletion cancelled.");
    }
}

fn handle_detailed_stats(habit_tracker: &HabitTracker) {
    println!("\n📈 === Detailed Habit Statistics ===");

    if habit_tracker.tracked_habits.is_empty() {
        println!("📝 No habits to analyze yet. Add some habits first!");
        return;
    }

    let total_tracking_days = habit_tracker.daily_habit_records.len();
    println!("📅 Total days tracked: {}", total_tracking_days);

    if let Some(earliest_date) = habit_tracker.daily_habit_records.first() {
        println!("🗓️  Tracking since: {}", earliest_date.record_date);
    }

    println!("\n🎯 Individual Habit Analysis:");
    for habit_name in &habit_tracker.tracked_habits {
        let (completed, total, percentage) = habit_tracker.calculate_habit_statistics(habit_name);

        println!("\n📊 {}", habit_name);
        println!("   Completed: {} days", completed);
        println!("   Total tracked: {} days", total);
        println!("   Success rate: {:.1}%", percentage);

        let streak = calculate_current_streak(habit_tracker, habit_name);
        if streak > 0 {
            println!("   Current streak: {} days 🔥", streak);
        }
    }
}

fn calculate_current_streak(habit_tracker: &HabitTracker, target_habit: &str) -> usize {
    let mut current_streak = 0;

    for daily_record in habit_tracker.daily_habit_records.iter().rev() {
        if let Some(habit_completion) = daily_record
            .habit_completions
            .iter()
            .find(|completion| completion.habit_name.to_lowercase() == target_habit.to_lowercase())
        {
            if habit_completion.is_completed {
                current_streak += 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    current_streak
}

fn prompt_for_initial_habits(habit_tracker: &mut HabitTracker) {
    println!("🎉 Welcome to your Personal Habit Tracker!");

    for habit_name in habit_tracker.tracked_habits.clone() {
        if ask_yes_no_question(&format!("Did you complete '{}' today", habit_name)) {
            if let Some(todays_record) = habit_tracker.get_most_recent_record_mut() {
                if let Some(habit_completion) = todays_record
                    .habit_completions
                    .iter_mut()
                    .find(|completion| completion.habit_name == habit_name)
                {
                    habit_completion.is_completed = true;
                }
            }
        }
    }
}

fn main() {
    let mut habit_tracker: HabitTracker = load_tracker_data();

    let todays_date = Local::now().format("%Y-%m-%d").to_string();
    let needs_new_daily_record = match habit_tracker.get_most_recent_date() {
        Some(last_recorded_date) => last_recorded_date != todays_date,
        None => true,
    };

    if needs_new_daily_record {
        habit_tracker.create_new_daily_record(todays_date);
        if !habit_tracker.tracked_habits.is_empty() {
            prompt_for_initial_habits(&mut habit_tracker);
        }
        if let Err(e) = save_tracker_data(&habit_tracker) {
            eprintln!("⚠️  Warning: {}", e);
        }
    }

    loop {
        display_main_menu();

        let user_menu_choice: UserChoice = match input!("\nWhat would you like to do? (1-6): ") {
            Ok(choice) => choice,
            Err(_) => {
                println!("❌ Please enter a valid number between 1 and 6");
                continue;
            }
        };

        match user_menu_choice {
            UserChoice::AddHabit => handle_add_habit(&mut habit_tracker),
            UserChoice::UpdateTodaysHabits => handle_update_habits(&mut habit_tracker),
            UserChoice::ViewProgress => handle_view_progress(&habit_tracker),
            UserChoice::DeleteHabit => handle_delete_habit(&mut habit_tracker),
            UserChoice::ViewDetailedStats => handle_detailed_stats(&habit_tracker),
            UserChoice::ExitProgram => {
                println!(
                    "👋 Thanks for using the Habit Tracker! Keep building those positive habits! 🌟"
                );
                break;
            }
        }

        if let Err(e) = save_tracker_data(&habit_tracker) {
            eprintln!("⚠️  Warning: {}", e);
        }
    }
}
