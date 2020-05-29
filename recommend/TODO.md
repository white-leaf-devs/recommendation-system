# TODO List

- [x] Manhattan distance 
- [x] Euclidean distance
- [x] Minkowski distance 
- [x] Pearson coefficient  
- [x] Cosine similarity
- [x] Jaccard index and distance
- [x] k nearest neighbors 
- [x] Rating prediction based on pearson and k-nn
- [x] For prediction select relevant users that rated the specified item for the knn
- [x] max_heap_knn and min_heap_knn should receive the queried MapedRatings in order to allow the above todos
- [x] Line 159, avoid querying the users and their ratings in each iteration, find a way to query all of the selected users and their ratings which should have the specified item (Group query and filter? probably modifying the controller interface)