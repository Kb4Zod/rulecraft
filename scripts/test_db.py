import sqlite3

conn = sqlite3.connect('rulecraft.db')
cursor = conn.cursor()

print("--- SPELLS ---")
cursor.execute("SELECT title, category FROM rules WHERE category = 'Spells' LIMIT 5;")
for row in cursor.fetchall():
    print(row)

print("--- EQUIPMENT ---")
cursor.execute("SELECT title, category FROM rules WHERE category = 'Equipment' LIMIT 5;")
for row in cursor.fetchall():
    print(row)

conn.close()
