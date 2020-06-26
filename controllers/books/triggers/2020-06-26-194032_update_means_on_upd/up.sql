-- Your SQL goes here

-- Create function to update mean
CREATE FUNCTION update_mean_on_upd() RETURNS TRIGGER AS
$BODY$
BEGIN
    UPDATE means 
    SET val = means.val + (new.score - old.score) / means.score_number;

    RETURN old;
END;
$BODY$
LANGUAGE plpgsql;

-- Create the trigger on insert
CREATE TRIGGER update_means_on_upd_rating
AFTER UPDATE ON ratings
FOR EACH ROW
EXECUTE FUNCTION update_mean_on_upd();