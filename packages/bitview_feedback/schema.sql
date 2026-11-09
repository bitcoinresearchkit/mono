-- The inbox: messages not yet read by the maintainer's job, which deletes each row as it takes it.
CREATE TABLE feedback (
  id      INTEGER PRIMARY KEY AUTOINCREMENT, -- never reused: the job keeps a message under its id
  day     TEXT NOT NULL,        -- UTC date, no time of day
  page    TEXT NOT NULL,        -- the app's folder, as /compass/
  link    TEXT NOT NULL,        -- '' unless the sender ticked to attach the exact address
  version TEXT NOT NULL,        -- the page's content hash (Bitview-Version), or ''
  browser TEXT NOT NULL,        -- chromium|firefox|safari|other, -, phone|desk
  message TEXT NOT NULL,
  contact TEXT NOT NULL,        -- '' or what the sender chose to give
  hash    TEXT NOT NULL UNIQUE  -- SHA-256 of page and message: an exact repeat while unread is one row
);
