import re
import copy

class Table:
    def __init__(self, name, matrix):
        self.name = name
        self.matrix = matrix

only_psql_file = open('tests/result_psql.out')
only_mongo_file = open('tests/result_mongo.out')
only_both_file = open('tests/result_psql_mongo.out')


psql_lines = only_psql_file.readlines()
mongo_lines = only_mongo_file.readlines()
both_lines = only_both_file.readlines()

datasets_results = []

matrix = []
for (psql_l, mongo_l, both_l) in zip(psql_lines, mongo_lines, both_lines):
    if 'Operation took' in psql_l:
        psql_time = re.search("[+-]?([0-9]*[.])?[0-9]+", psql_l).group()
        mongo_time = re.search("[+-]?([0-9]*[.])?[0-9]+", mongo_l).group()
        both_time = re.search("[+-]?([0-9]*[.])?[0-9]+", both_l).group()
        matrix.append([psql_time, mongo_time, both_time])
    elif 'Disconnecting' in psql_l:
        datasets_results.append(copy.deepcopy(matrix))
        matrix = []

queries_file = open('tests/test.in')
queries_lines = queries_file.readlines()

current_db_index = -1
name = ''
current_knn_tests = 0
current_user_tests = 0
current_item_tests = 0

knn_tables, user_tables, item_tables = [], [], []

for line in queries_lines:
    if 'disconnect' in line:
        knn_table = datasets_results[current_db_index][0:current_knn_tests]
        user_table = datasets_results[current_db_index][current_knn_tests:current_knn_tests+current_user_tests]
        item_table = datasets_results[current_db_index][current_knn_tests+current_user_tests:current_knn_tests+current_user_tests+current_item_tests]
        knn_tables.append(Table(name+' KNN', knn_table))
        user_tables.append(Table(name+' USER', user_table))
        item_tables.append(Table(name+' ITEM', item_table))

        name = ''
        current_knn_tests = 0
        current_user_tests = 0
        current_item_tests = 0
    elif 'connect' in line:
        name = line[8:len(line)-2]
        current_db_index += 1
    elif 'user_knn(' in line:
        current_knn_tests += 1
    elif 'user_based' in line:
        current_user_tests += 1
    elif 'item_based' in line:
        current_item_tests += 1
        

for table in knn_tables:
    print(table.name)
    for row in table.matrix:
        for item in row:
            print(float(item),',',end='')
        print()

print()

for table in user_tables:
    print(table.name)
    for row in table.matrix:
        for item in row:
            print(float(item),',',end='')
        print()

print()

for table in item_tables:
    print(table.name)
    for row in table.matrix:
        for item in row:
            print(float(item),',',end='')
        print()