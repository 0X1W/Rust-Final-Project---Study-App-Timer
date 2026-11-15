Elementary System Programming | Final Project

Team HMP - Study App Timer Project

Members:

69011442 - Peerapat Pooncharoen
  - UI developer
  - Theme changes
  - Misc. features handling and fn update()
  - fn view()

69011443 - Peeraphad Khongchoom
  - StudyTimer Struct
  - Timer countdown, modes, setting changes logic/code
  - Most fn update() code
  - Unit testing
    
69011497 - Ruharn Audthanont
  - File handling for saving and loading
  - Formatting session content
  - Error handling
  - User settings changes


Features:
  - Pomodoro Technique Study App: In default settings, focus study for 25 minutes, short break for 5 minutes, and then long break for 15 minutes every 4 intervals.
  - Countdown timer for studying
  - Input session name and your own name to save as history
  - File saved as txt
  - View session history
  - Change study settings and theme

OS: Windows

How to run: cargo run

<img width="1022" height="796" alt="image" src="https://github.com/user-attachments/assets/0007e146-1aea-4587-a016-34af0a2bff2d" />
<img width="1018" height="793" alt="image" src="https://github.com/user-attachments/assets/9c9bebbe-79a9-45a4-a286-56c42f5cb0c4" />
<img width="1020" height="756" alt="image" src="https://github.com/user-attachments/assets/6459201d-88bb-48a8-bfcc-63fb56f03ad2" />
<img width="1022" height="796" alt="image" src="https://github.com/user-attachments/assets/0adb34e0-297f-4bb7-9480-7d9c56d743db" />

What we learned:
  - Working in groups
  - Remote working
  - How to develop rust ui that updates in real time without user input (Applicantion)
  - How to develop countdown timer
  - Handle multiple screens
  - I learn to load and save data in txt file, while access the said data and format it and handling the error.
  - I also learn to make data interact with the GUI and at last I learn to fight with the "Compile error".

Problems encountered:
  - Clock timer not properly syncing
  - Timer not counting down properly (sometimes it takes longer than 1 second to count down)
  - Putting non numbers in the settings
  - Calculating session time
  - UI not updating when change the time in the settings
  - syntax error, trying to or figuring the way to save data while having the input to the text file corrupted (I mean save in a non-desired way), I also had a problem when I pull out the data and have to format it.
