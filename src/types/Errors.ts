export interface UniversityErrorConstructorParams {
    userFault: boolean;
    message: string;
}

export default class UniversityError extends Error {

    //#region Properties
    private userFault: boolean;
    //#endregion

    //#region Constructor
    constructor(params: UniversityErrorConstructorParams) {
        super(params.message);
        this.name = "UniversityError";
        this.userFault = params.userFault;
    }
    //#endregion

    //#region Getters/Setters
    get UserFault(): boolean {
        return this.userFault;
    }
    //#endregion

}