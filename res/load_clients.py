import sqlite3

class Client:
	def __init__(self, fields):
		self.name = fields[0]
		self.surname = fields[1]
		self.country = fields[2]
		self.city = fields[3]
		self.balance = fields[4]
		self.password = fields[5]
		self.email = fields[6]

	def insert(self, cur):
		sql = f"insert into Client values ('{self.name}', '{self.surname}', '{self.country}', '{self.city}', {self.balance}, '{self.password}', '{self.email}')"
		cur.execute(sql)

if __name__ == '__main__':
	with sqlite3.connect("clients.db") as con:
		cur = con.cursor()
		cur.execute(
			"""
			create table Client(name text,surname text,country text,city text,balance real,password text,email text);
			"""
		)
		with open("clients.csv", "r") as fs:
			headers = fs.readline()
			count = 0
			for line in fs:
				client = Client(line.split(','))
				client.insert(cur)
				count += 1

		con.commit()
		print("Loaded", count, "lines into database !")