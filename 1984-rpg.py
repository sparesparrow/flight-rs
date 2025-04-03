import random
import time
import os

class Character:
    def __init__(self, name, occupation):
        self.name = name
        self.occupation = occupation
        self.loyalty = 50  # Loyalty to the Party (0-100)
        self.suspicion = 0  # How suspicious the Party is of you (0-100)
        self.thoughtcrime = 0  # Level of thoughtcrime (0-100)
        self.health = 100
        self.inventory = []
        self.relationships = {}  # People you know and trust levels
        self.location = "Victory Mansions"
        self.journal_entries = []
        self.tasks_completed = 0
        self.rebellion_score = 0

    def display_stats(self):
        """Display character statistics"""
        print("\n" + "=" * 40)
        print(f"Name: {self.name} | Occupation: {self.occupation}")
        print(f"Location: {self.location}")
        print("-" * 40)
        print(f"Health: {self.health}/100")
        print(f"Party Loyalty: {self.loyalty}/100")
        print(f"Suspicion Level: {self.suspicion}/100")
        print(f"Thoughtcrime Level: {self.thoughtcrime}/100")
        print(f"Rebellion Score: {self.rebellion_score}/100")
        print(f"Tasks Completed: {self.tasks_completed}")
        print("=" * 40)

    def write_journal(self, entry):
        """Add an entry to your secret journal"""
        self.journal_entries.append(entry)
        self.thoughtcrime += 5
        print("\nYou've written in your journal. Thoughtcrime level increased.")
        
    def view_journal(self):
        """View your journal entries"""
        if not self.journal_entries:
            print("\nYour journal is empty.")
            return
        
        print("\n==== YOUR SECRET JOURNAL ====")
        for i, entry in enumerate(self.journal_entries, 1):
            print(f"\nEntry {i}:")
            print(f"{entry}")
        print("\n============================")
        
        # Risk of being caught
        if random.randint(1, 10) == 1:
            self.suspicion += 10
            print("\nYou feel like you were being watched while reading your journal.")
            print("Suspicion increased by 10!")

class World:
    def __init__(self):
        self.locations = {
            "Victory Mansions": {
                "description": "Your dilapidated apartment building. The telescreen on the wall continuously broadcasts Party propaganda.",
                "connections": ["Ministry of Truth", "Victory Square"],
                "safety": 3,  # 1-5 scale (5 is safest)
            },
            "Ministry of Truth": {
                "description": "A massive pyramidal structure where you work rewriting historical documents to match Party narratives.",
                "connections": ["Victory Mansions", "Victory Square", "Canteen"],
                "safety": 1,
            },
            "Canteen": {
                "description": "A gray cafeteria serving tasteless Victory meals and Victory Gin.",
                "connections": ["Ministry of Truth"],
                "safety": 2,
            },
            "Victory Square": {
                "description": "The central square where public executions and rallies are held.",
                "connections": ["Victory Mansions", "Ministry of Truth", "Prole District", "Charrington's Shop"],
                "safety": 1,
            },
            "Prole District": {
                "description": "The rundown area where the proles (working class) live with less surveillance.",
                "connections": ["Victory Square", "Charrington's Shop"],
                "safety": 4,
            },
            "Charrington's Shop": {
                "description": "An antique shop run by an elderly man. It has a room upstairs without a telescreen.",
                "connections": ["Victory Square", "Prole District"],
                "safety": 3,
            },
            "Ministry of Love": {
                "description": "The terrifying windowless building where enemies of the Party are taken. Room 101 is inside.",
                "connections": [],  # No escape
                "safety": 0,
            },
        }
        
        self.npcs = {
            "O'Brien": {
                "description": "A high-ranking Inner Party member who seems to have rebellious tendencies.",
                "trust": 0,  # Will betray you
                "location": "Ministry of Truth",
            },
            "Julia": {
                "description": "A young woman who works in the Fiction Department of the Ministry of Truth.",
                "trust": 80,
                "location": "Ministry of Truth",
            },
            "Charrington": {
                "description": "The seemingly friendly old man who runs the antique shop.",
                "trust": 0,  # Thought Police agent
                "location": "Charrington's Shop",
            },
            "Parsons": {
                "description": "Your neighbor, an enthusiastic Party supporter whose children spy on adults.",
                "trust": 20,
                "location": "Victory Mansions",
            },
            "Syme": {
                "description": "A philologist working on the 11th edition of the Newspeak dictionary.",
                "trust": 50,
                "location": "Canteen",
            },
        }
        
        self.current_date = "April 4th, 1984"
        self.two_minutes_hate_today = True
        self.chocolate_ration = 20  # Will be reduced to 15, then announced as increased
        self.current_enemy = "Eurasia"  # Will change to Eastasia
        
    def display_location(self, location):
        """Display information about the current location"""
        loc = self.locations[location]
        print(f"\n=== {location} ===")
        print(loc["description"])
        
        # Show available connections
        print("\nYou can go to:")
        for connection in loc["connections"]:
            print(f"- {connection}")
            
        # Show NPCs at this location
        present_npcs = [name for name, info in self.npcs.items() if info["location"] == location]
        if present_npcs:
            print("\nPeople present:")
            for npc in present_npcs:
                print(f"- {npc}")
                
        # Random event chance based on safety
        self._random_event(location)
    
    def _random_event(self, location):
        """Chance of random events based on location safety"""
        safety = self.locations[location]["safety"]
        if random.randint(1, 10) > safety:
            events = [
                "A patrol of Thought Police officers walks by, scanning faces.",
                "The telescreen suddenly announces a reduction in the chocolate ration.",
                "Everyone around you freezes as an announcement begins about a captured traitor.",
                "You notice someone watching you intently before they quickly look away.",
                "A child in a Youth League uniform points at a man who is promptly arrested.",
                "Party members gather for an impromptu Two Minutes Hate.",
            ]
            print(f"\n[EVENT] {random.choice(events)}")


class Game:
    def __init__(self):
        self.world = World()
        self.player = None
        self.game_over = False
        self.day = 1
        self.tutorial_done = False
        
    def clear_screen(self):
        """Clear the console screen"""
        os.system('cls' if os.name == 'nt' else 'clear')
        
    def slow_print(self, text, delay=0.03):
        """Print text slowly for dramatic effect"""
        for char in text:
            print(char, end='', flush=True)
            time.sleep(delay)
        print()
        
    def intro(self):
        """Display game introduction"""
        self.clear_screen()
        self.slow_print("\n" + "=" * 60, 0.01)
        self.slow_print("1 9 8 4: S H A D O W S  O F  O C E A N I A", 0.05)
        self.slow_print("=" * 60, 0.01)
        self.slow_print("\nWAR IS PEACE | FREEDOM IS SLAVERY | IGNORANCE IS STRENGTH", 0.05)
        self.slow_print("\nWelcome to Oceania, citizen. The year is 1984.", 0.03)
        self.slow_print("Big Brother is watching. The Thought Police are listening.", 0.03)
        self.slow_print("Be careful what you think. Be careful what you say.", 0.03)
        self.slow_print("The wrong thought could be your last...", 0.03)
        input("\nPress Enter to continue...")
        
        self.clear_screen()
        self.slow_print("\nYou wake up to the harsh sound of the telescreen.")
        self.slow_print("\"Citizens of Oceania! The chocolate ration has been increased to 20 grams per week!\"")
        self.slow_print("You remember clearly that it was 30 grams last week. But you say nothing.")
        self.slow_print("Saying nothing is how you survive.")
        
    def create_character(self):
        """Character creation process"""
        self.clear_screen()
        print("\n=== CHARACTER CREATION ===")
        print("Who are you in the world of Oceania?")
        
        name = input("\nEnter your name: ").strip()
        
        print("\nSelect your occupation:")
        print("1. Records Department Worker (Ministry of Truth)")
        print("2. Maintenance Technician (Ministry of Plenty)")
        print("3. Junior Spy Instructor (Ministry of Love)")
        print("4. Fiction Department Writer (Ministry of Truth)")
        
        while True:
            choice = input("\nEnter choice (1-4): ")
            if choice == "1":
                occupation = "Records Department Worker"
                break
            elif choice == "2":
                occupation = "Maintenance Technician"
                break
            elif choice == "3":
                occupation = "Junior Spy Instructor"
                break
            elif choice == "4":
                occupation = "Fiction Department Writer"
                break
            else:
                print("Invalid choice. Please select 1-4.")
        
        self.player = Character(name, occupation)
        
        # Adjust starting stats based on occupation
        if occupation == "Records Department Worker":
            self.player.loyalty -= 5  # Working with changing history makes you question things
            self.player.thoughtcrime += 10
        elif occupation == "Junior Spy Instructor":
            self.player.loyalty += 15  # You're deeply involved in Party structure
            self.player.suspicion -= 10
        elif occupation == "Fiction Department Writer":
            self.player.thoughtcrime += 15  # Creative thinking leads to dangerous thoughts
            
        print(f"\nWelcome, {name}. Your life as a {occupation} is about to change...")
        self.player.display_stats()
        input("\nPress Enter to begin your journey...")
    
    def tutorial(self):
        """Game tutorial"""
        self.clear_screen()
        print("\n=== GAME TUTORIAL ===")
        print("\nIn 1984: Shadows of Oceania, you navigate a dangerous world where thinking the wrong thing can get you killed.")
        print("\nKey Concepts:")
        print("- Loyalty: Your commitment to the Party. Too low and you'll be suspected.")
        print("- Suspicion: How much the Party doubts you. Reach 100 and you'll be arrested.")
        print("- Thoughtcrime: Your level of rebellious thinking. Affects your actions and decisions.")
        print("- Health: Your physical condition. Reaches 0 and you die.")
        print("- Rebellion: Your progress toward meaningful resistance (if possible).")
        
        print("\nEach day, you will:")
        print("1. Perform your work duties")
        print("2. Navigate locations in Oceania")
        print("3. Interact with other characters")
        print("4. Make choices that affect your stats and story")
        
        print("\nRemember: In this world, there is no concept of winning in the traditional sense.")
        print("Your goal is to survive, maintain your humanity, and perhaps find small acts of rebellion.")
        print("Or perhaps total submission to the Party is the only way to survive...")
        
        print("\nGood luck. You'll need it.")
        input("\nPress Enter to start Day 1...")
        self.tutorial_done = True
    
    def main_menu(self):
        """Display main menu"""
        self.clear_screen()
        print(f"\n=== DAY {self.day}: {self.world.current_date} ===")
        print(f"Current Enemy of Oceania: {self.world.current_enemy}")
        
        self.player.display_stats()
        self.world.display_location(self.player.location)
        
        print("\n=== ACTIONS ===")
        print("1. Move to another location")
        print("2. Interact with someone here")
        print("3. Work on daily tasks")
        print("4. Search for items")
        print("5. Write in journal")
        print("6. View journal")
        print("7. Rest until tomorrow")
        print("8. Game information")
        print("9. Quit game")
        
        choice = input("\nWhat will you do? ")
        
        if choice == "1":
            self.move_location()
        elif choice == "2":
            self.interact_with_npc()
        elif choice == "3":
            self.do_work()
        elif choice == "4":
            self.search_items()
        elif choice == "5":
            self.write_in_journal()
        elif choice == "6":
            self.player.view_journal()
            input("\nPress Enter to continue...")
        elif choice == "7":
            self.next_day()
        elif choice == "8":
            self.show_help()
        elif choice == "9":
            self.quit_game()
        else:
            print("Invalid choice. Try again.")
            input("\nPress Enter to continue...")
    
    def move_location(self):
        """Move to a different location"""
        current = self.player.location
        connections = self.world.locations[current]["connections"]
        
        print("\n=== TRAVEL ===")
        print(f"You are currently at {current}.")
        print("Where would you like to go?")
        
        for i, location in enumerate(connections, 1):
            print(f"{i}. {location}")
        print(f"{len(connections) + 1}. Stay at {current}")
        
        while True:
            try:
                choice = int(input("\nEnter choice: "))
                if 1 <= choice <= len(connections):
                    new_location = connections[choice - 1]
                    
                    # Travel risk check
                    if self.player.suspicion > 70 and random.randint(1, 10) <= 3:
                        print("\nAs you're traveling, you notice you're being followed by someone in a black coat.")
                        print("Your suspicion level is very high. Be careful.")
                        self.player.suspicion += 5
                    
                    self.player.location = new_location
                    print(f"\nYou travel to {new_location}.")
                    break
                elif choice == len(connections) + 1:
                    print(f"\nYou decide to stay at {current}.")
                    break
                else:
                    print("Invalid choice. Try again.")
            except ValueError:
                print("Please enter a number.")
        
        input("\nPress Enter to continue...")
    
    def interact_with_npc(self):
        """Interact with NPCs at the current location"""
        location = self.player.location
        present_npcs = [name for name, info in self.world.npcs.items() if info["location"] == location]
        
        if not present_npcs:
            print("\nThere's no one here to interact with.")
            input("\nPress Enter to continue...")
            return
            
        print("\n=== INTERACT ===")
        print("Who would you like to speak with?")
        
        for i, npc in enumerate(present_npcs, 1):
            print(f"{i}. {npc}")
        print(f"{len(present_npcs) + 1}. No one")
        
        while True:
            try:
                choice = int(input("\nEnter choice: "))
                if 1 <= choice <= len(present_npcs):
                    npc_name = present_npcs[choice - 1]
                    self.npc_interaction(npc_name)
                    break
                elif choice == len(present_npcs) + 1:
                    print("\nYou decide not to speak with anyone.")
                    break
                else:
                    print("Invalid choice. Try again.")
            except ValueError:
                print("Please enter a number.")
                
        input("\nPress Enter to continue...")
    
    def npc_interaction(self, npc_name):
        """Handle interaction with a specific NPC"""
        npc = self.world.npcs[npc_name]
        
        print(f"\n=== SPEAKING WITH {npc_name.upper()} ===")
        print(npc["description"])
        
        # Different dialogue options based on NPC
        if npc_name == "O'Brien":
            self.obrien_interaction()
        elif npc_name == "Julia":
            self.julia_interaction()
        elif npc_name == "Charrington":
            self.charrington_interaction()
        elif npc_name == "Parsons":
            self.parsons_interaction()
        elif npc_name == "Syme":
            self.syme_interaction()
    
    def obrien_interaction(self):
        """Interaction with O'Brien"""
        print("\nO'Brien nods to you with a slight smile.")
        print("\"Ah, comrade. How goes your work for the Party?\"")
        
        print("\n1. \"Very well. Glory to Big Brother.\" (Safe response)")
        print("2. \"I sometimes find it... difficult.\" (Risky response)")
        print("3. \"I have questions about the Brotherhood...\" (Dangerous response)")
        
        choice = input("\nYour response: ")
        
        if choice == "1":
            print("\nO'Brien nods approvingly.")
            print("\"The Party appreciates your dedication, comrade.\"")
            self.player.loyalty += 5
            print("Loyalty increased by 5.")
        elif choice == "2":
            print("\nO'Brien's eyes narrow slightly, then he smiles.")
            print("\"We all have our burdens to bear for the Party. Perhaps we should discuss this further sometime.\"")
            if self.player.thoughtcrime > 30:
                print("He subtly hands you a note with his address.")
                self.player.inventory.append("O'Brien's address")
                print("'O'Brien's address' added to inventory.")
            self.player.suspicion += 5
            print("Suspicion increased by 5.")
        elif choice == "3":
            print("\nO'Brien's face becomes completely blank.")
            print("\"I'm not sure I understand what you're referring to, comrade.\"")
            print("He walks away, and you notice someone watching you from across the room.")
            self.player.suspicion += 15
            print("Suspicion increased by 15!")
            
            # Potentially harmful consequence
            if random.randint(1, 3) == 1:
                print("\nLater that day, you're called in for questioning about your work.")
                print("The interrogator seems particularly interested in your political views.")
                self.player.suspicion += 10
                print("Suspicion increased by another 10!")
        else:
            print("\nYou mumble something incoherent.")
            print("O'Brien gives you a strange look and walks away.")
            
    def julia_interaction(self):
        """Interaction with Julia"""
        # This will be implemented with romance options and resistance plotting
        print("\n\"Hello,\" Julia says, glancing around to see if anyone is watching.")
        
        print("\n1. Greet her formally, as a good Party member should")
        print("2. Pass her a secret note")
        print("3. Suggest meeting somewhere private")
        
        choice = input("\nYour response: ")
        
        if choice == "1":
            print("\nYou greet Julia with the proper Party salute.")
            print("She seems disappointed but returns the gesture perfectly.")
            self.player.loyalty += 5
            print("Loyalty increased by 5.")
        elif choice == "2":
            if "Julia's trust" not in self.player.relationships:
                print("\nYou discreetly slip her a note. She takes it without looking at you.")
                print("Later, you find a note in your pocket: \"Prole District, tomorrow.\"")
                self.player.relationships["Julia's trust"] = 10
                print("You've established a connection with Julia.")
                self.player.thoughtcrime += 10
                self.player.suspicion += 5
                print("Thoughtcrime increased by 10.")
                print("Suspicion increased by 5.")
            else:
                print("\nShe takes your note and passes one back.")
                print("\"Charrington's Shop, upstairs room. Three days from now.\"")
                self.player.relationships["Julia's trust"] += 10
                self.player.thoughtcrime += 5
                print("Your relationship with Julia has deepened.")
                print("Thoughtcrime increased by 5.")
        elif choice == "3":
            if "Julia's trust" in self.player.relationships and self.player.relationships["Julia's trust"] > 20:
                print("\nYou whisper about meeting somewhere without telescreens.")
                print("She nods slightly and whispers back, \"I know a place. Follow me later.\"")
                
                print("\nYou spend several precious hours with Julia, away from the Party's eyes.")
                print("For a brief moment, you both feel truly human again.")
                self.player.health += 10
                self.player.thoughtcrime += 15
                self.player.rebellion_score += 5
                print("Health improved by 10.")
                print("Thoughtcrime increased by 15.")
                print("Rebellion score increased by 5.")
            else:
                print("\nShe looks alarmed at your suggestion.")
                print("\"I don't know what you mean, comrade. We should all be where the Party needs us.\"")
                print("She walks away quickly. You realize you've made a mistake.")
                self.player.suspicion += 10
                print("Suspicion increased by 10!")
        else:
            print("\nYou say nothing coherent. Julia gives you an odd look and walks away.")
    
    def charrington_interaction(self):
        """Interaction with Mr. Charrington"""
        print("\nThe old shopkeeper smiles warmly at you.")
        print("\"Looking for any particular antiques today?\"")
        
        print("\n1. \"Just browsing, thank you.\"")
        print("2. \"Do you have any items from before the Revolution?\"")
        print("3. \"I'm interested in the room upstairs.\"")
        
        choice = input("\nYour response: ")
        
        if choice == "1":
            print("\nCharrington nods. \"Take your time, plenty to see.\"")
            print("You browse the dusty shelves filled with useless trinkets.")
            
            # Chance to find something
            if random.randint(1, 3) == 1:
                print("\nYou notice a small coral paperweight with a piece of coral inside.")
                print("It seems to be a beautiful relic from the past.")
                
                print("\n1. Purchase it")
                print("2. Leave it be")
                
                subchoice = input("\nYour choice: ")
                if subchoice == "1":
                    print("\nYou buy the paperweight. It reminds you of a world before the Party.")
                    self.player.inventory.append("Coral paperweight")
                    self.player.thoughtcrime += 5
                    print("'Coral paperweight' added to inventory.")
                    print("Thoughtcrime increased by 5.")
                else:
                    print("\nYou decide it's safer not to possess such items.")
            
        elif choice == "2":
            print("\nCharrington's eyes light up.")
            print("\"Oh yes, a few things here and there. The Party doesn't mind these old trinkets.\"")
            
            if "Charrington trust" not in self.player.relationships:
                self.player.relationships["Charrington trust"] = 5
                print("You've established a rapport with Mr. Charrington.")
            else:
                self.player.relationships["Charrington trust"] += 5
                print("Your relationship with Mr. Charrington has improved.")
                
            print("\nHe shows you a few items, including an old rhyme book.")
            self.player.thoughtcrime += 5
            self.player.suspicion += 5
            print("Thoughtcrime increased by 5.")
            print("Suspicion increased by 5.")
            
        elif choice == "3":
            if "Julia's trust" in self.player.relationships and self.player.relationships["Julia's trust"] > 30:
                print("\nCharrington smiles knowingly.")
                print("\"Ah yes, a quiet place. No telescreens up there. Two dollars a week.\"")
                print("\nYou arrange to rent the room. It could be a sanctuary from the Party's eyes.")
                self.player.inventory.append("Key to upstairs room")
                self.player.thoughtcrime += 10
                self.player.suspicion += 10
                self.player.rebellion_score += 10
                print("'Key to upstairs room' added to inventory.")
                print("Thoughtcrime increased by 10.")
                print("Suspicion increased by 10.")
                print("Rebellion score increased by 10.")
                
                # What you don't know is that the room is bugged...
            else:
                print("\nCharrington looks confused.")
                print("\"The upstairs? Just storage, nothing interesting there.\"")
                print("He seems suspicious of your question.")
                self.player.suspicion += 5
                print("Suspicion increased by 5.")
        else:
            print("\nYou mumble something and Charrington nods politely.")
    
    def parsons_interaction(self):
        """Interaction with Parsons"""
        print("\nParsons greets you with excessive enthusiasm.")
        print("\"Comrade! Have you contributed to the Hate Week preparations yet?\"")
        
        print("\n1. \"Of course! Glory to Big Brother!\"")
        print("2. \"I've been too busy with work.\"")
        print("3. \"I'm not interested in Hate Week.\"")
        
        choice = input("\nYour response: ")
        
        if choice == "1":
            print("\nParsons beams with approval.")
            print("\"That's the spirit! My little ones are so excited they're practicing their spying techniques!\"")
            print("He laughs, not realizing how terrifying that sounds.")
            self.player.loyalty += 5
            print("Loyalty increased by 5.")
            
            if random.randint(1, 4) == 1:
                print("\nParsons lowers his voice. \"Between you and me, I talk in my sleep sometimes.")
                print("My little girl reported me for saying 'Down with Big Brother' in my sleep last week!\"")
                print("He seems proud of his daughter's vigilance, unaware of his danger.")
                self.player.thoughtcrime += 5
                print("Thoughtcrime increased by 5.")
                
        elif choice == "2":
            print("\nParsons looks disappointed.")
            print("\"Too busy? No one's too busy for the Party! I'll put your name down for extra duty!\"")
            print("Before you can protest, he's already making a note in his book.")
            self.player.loyalty += 10
            self.player.health -= 5
            print("Loyalty increased by 10.")
            print("Health decreased by 5 due to extra work.")
            
        elif choice == "3":
            print("\nParsons looks shocked, then laughs nervously.")
            print("\"That's a good joke, comrade! Of course you're interested, we all are!\"")
            print("He walks away but glances back at you with concern.")
            self.player.suspicion += 15
            print("Suspicion increased by 15!")
            
        else:
            print("\nYou give a noncommittal response. Parsons seems satisfied enough.")
    
    def syme_interaction(self):
        """Interaction with Syme"""
        print("\nSyme is excited to tell you about his work on the Newspeak dictionary.")
        print("\"We're removing thousands more words this year! Thoughtcrime will be literally impossible!\"")
        
        print("\n1. Express enthusiasm for his work")
        print("2. Ask technical questions about Newspeak")
        print("3. Question if limiting language limits human experience")
        
        choice = input("\nYour response: ")
        
        if choice == "1":
            print("\nYou tell Syme his work is vital to the Party's goals.")
            print("He nods eagerly. \"The Eleventh Edition will be perfect! No more unnecessary words!\"")
            self.player.loyalty += 5
            print("Loyalty increased by 5.")
        elif choice == "2":
            print("\nYou ask Syme about the technical aspects of vocabulary reduction.")
            print("He launches into a passionate explanation of how they're eliminating synonyms.")
            print("\"Why have 'excellent', 'splendid', and 'great' when 'plusgood' and 'doubleplusgood' suffice?\"")
            
            self.player.thoughtcrime += 5
            print("Thoughtcrime increased by 5 - his explanation makes you realize what's being lost.")
            
            # Foreshadowing
            print("\nAs Syme talks, you realize he understands too well what the Party is doing.")
            print("You remember that people who understand too much often disappear...")
            
        elif choice == "3":
            print("\nYou carefully ask if reducing vocabulary might limit certain types of thought.")
            print("\nSyme stares at you intensely. \"That's precisely the point, don't you see?")
            print("We're making thoughtcrime impossible because there will be no words to express it!\"")
            
            print("\nHis candor is frightening. He's too intelligent, too perceptive about the Party's goals.")
            print("You're certain Syme will be vaporized eventually, despite his loyalty.")
            
            self.player.thoughtcrime += 10
            self.player.suspicion += 5
            print("Thoughtcrime increased by 10.")
            print("Suspicion increased by 5.")
            
        else:
            print("\nYou nod along without saying much. Syme eventually finds someone else to talk to.")
    
    def do_work(self):
        """Complete daily work tasks"""
        print("\n=== WORK TASKS ===")
        
        # Different tasks based on occupation
        if self.player.occupation == "Records Department Worker":
            self.records_department_work()
        elif self.player.occupation == "Maintenance Technician":
            self.maintenance_work()
        elif self.player.occupation == "Junior Spy Instructor":
            self.spy_instructor_work()
        elif self.player.occupation == "Fiction Department Writer":
            self.fiction_department_work()
            
        # Check if task completed today
        if self.player.location == "Ministry of Truth" or self.player.location == "Ministry of Love":
            self.player.tasks_completed += 1
            
            # Sometimes the Party changes history
            if random.randint(1, 5) == 1:
                self.party_changes_history()