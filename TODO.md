# TODO
* Frontend works only without server, syncing notes seems to be broken -> FIX

* Implement tags
* Show local datetime on frontend using js-sys

# LONGTERM
* Handle errors/options/results better

# DONE
* When editing, delete button deletes correct note, but saving it overwrites note under
* Editing with DB connected is not working 
* Figure how to add timestamp on frontend (now it's complicated) -> js-sys
* Timestamp note in backend
* Changes save using "method" field in note item and periodically send 1 JSON back to server to sync database
* On backend create 1 POST endpoint, that will sort updates from frontend
* Use localstorage, implement CREATE, UPDATE and DELETE on frontend first